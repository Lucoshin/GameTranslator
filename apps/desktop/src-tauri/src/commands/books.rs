use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use game_translator_app_core::{
    PerformanceSettings, ProviderFactory, ProviderSettings, WindowsCredentialStore,
};
use game_translator_content_book::{
    export_docx, export_epub, export_markdown, export_pdf, parse_book, BookExportFormat,
    BookExportProfile, BookExportRecord, BookProject, SegmentStatus,
};
use game_translator_qa_core::check_translation;
use game_translator_translation_core::{RunControl, TranslationOrchestrator, TranslationSegment};
use serde::Deserialize;
use tauri::Manager;

use super::{
    dto::{LanguageInput, ProviderInput},
    events::emit_progress,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TranslateBookInput {
    run_id: String,
    project: BookProject,
    chapter_id: Option<String>,
    provider: ProviderInput,
    source_language: LanguageInput,
    target_language: LanguageInput,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ExportBookInput {
    project: BookProject,
    format: BookExportFormat,
    profile: BookExportProfile,
}

fn books_directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map(|path| path.join("books"))
        .map_err(|error| error.to_string())
}

fn project_path(directory: &Path, project_id: &str) -> PathBuf {
    directory.join(format!("{project_id}.json"))
}

fn export_history_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map(|path| path.join("book-export-history.json"))
        .map_err(|error| error.to_string())
}

fn write_project(directory: &Path, project: &BookProject) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| format!("创建书籍项目目录失败：{error}"))?;
    let json = serde_json::to_vec_pretty(project)
        .map_err(|error| format!("序列化书籍项目失败：{error}"))?;
    fs::write(project_path(directory, &project.id), json)
        .map_err(|error| format!("保存书籍项目失败：{error}"))
}

fn read_projects(directory: &Path) -> Result<Vec<BookProject>, String> {
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut projects = Vec::new();
    for entry in
        fs::read_dir(directory).map_err(|error| format!("读取书籍项目目录失败：{error}"))?
    {
        let entry = entry.map_err(|error| format!("读取书籍项目失败：{error}"))?;
        if entry.path().extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(entry.path()).map_err(|error| format!("读取书籍项目失败：{error}"))?;
        let project: BookProject = serde_json::from_slice(&bytes)
            .map_err(|error| format!("书籍项目数据损坏（{}）：{error}", entry.path().display()))?;
        projects.push(project);
    }
    projects.sort_by(|left, right| left.title.cmp(&right.title));
    Ok(projects)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(super) fn import_book_project(app: tauri::AppHandle) -> Result<BookProject, String> {
    let path = rfd::FileDialog::new()
        .set_title("导入书籍")
        .add_filter("常见书籍", &["txt", "md", "markdown", "epub", "docx"])
        .pick_file()
        .ok_or_else(|| "未选择书籍文件".to_owned())?;
    let project = parse_book(&path).map_err(|error| error.to_string())?;
    write_project(&books_directory(&app)?, &project)?;
    Ok(project)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(super) fn list_book_projects(app: tauri::AppHandle) -> Result<Vec<BookProject>, String> {
    read_projects(&books_directory(&app)?)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(super) fn save_book_project(app: tauri::AppHandle, project: BookProject) -> Result<(), String> {
    write_project(&books_directory(&app)?, &project)
}

#[tauri::command]
pub(super) async fn translate_book_project(
    app: tauri::AppHandle,
    input: TranslateBookInput,
) -> Result<BookProject, String> {
    tauri::async_runtime::spawn_blocking(move || translate_book(&app, input))
        .await
        .map_err(|error| format!("书籍翻译任务异常终止：{error}"))?
}

fn translate_book(
    app: &tauri::AppHandle,
    input: TranslateBookInput,
) -> Result<BookProject, String> {
    let TranslateBookInput {
        run_id,
        mut project,
        chapter_id,
        provider,
        source_language,
        target_language,
    } = input;
    let skip_translated = chapter_id.is_none();
    let segments = segments_for_translation(&project, chapter_id.as_deref(), skip_translated);
    if segments.is_empty() {
        return Err("当前范围没有需要翻译的段落".to_owned());
    }

    emit_progress(
        app,
        &run_id,
        "translating",
        0,
        segments.len(),
        0,
        0,
        0,
        "正在翻译书稿",
    );
    let credentials = WindowsCredentialStore::new("GameTranslator");
    let translation_provider = ProviderFactory::new(&credentials)
        .create(&ProviderSettings::new(
            &provider.kind,
            &provider.base_url,
            &provider.model,
        ))
        .map_err(|error| error.to_string())?;
    let performance = PerformanceSettings::for_provider(
        &provider.kind,
        provider.performance.as_deref().unwrap_or("balanced"),
    );
    let orchestrator = TranslationOrchestrator::new(
        translation_provider.as_ref(),
        &provider.model,
        language_prompt(&source_language),
        language_prompt(&target_language),
        performance.batch_size,
    )
    .with_batch_character_budget(performance.character_budget)
    .with_adaptive_concurrency(performance.initial_concurrency, performance.max_concurrency);
    let total = segments.len();
    let result = orchestrator.run_with_progress(
        &segments,
        &HashMap::new(),
        RunControl::Running,
        |current| {
            emit_progress(
                app,
                &run_id,
                "translating",
                current.translations.len() + current.failed_segment_ids.len(),
                total,
                current.failed_segment_ids.len(),
                0,
                0,
                "正在翻译书稿",
            );
        },
    );

    let issues = apply_translations(&mut project, &result.translations);
    project.source_language = source_language.code;
    project.target_language = target_language.code;
    write_project(&books_directory(app)?, &project)?;
    emit_progress(
        app,
        &run_id,
        "completed",
        result.translations.len() + result.failed_segment_ids.len(),
        total,
        result.failed_segment_ids.len(),
        issues,
        0,
        "书稿翻译完成",
    );
    Ok(project)
}

fn segments_for_translation(
    project: &BookProject,
    chapter_id: Option<&str>,
    skip_translated: bool,
) -> Vec<TranslationSegment> {
    project
        .chapters
        .iter()
        .filter(|chapter| match chapter_id {
            Some(id) => id == chapter.id,
            None => true,
        })
        .flat_map(|chapter| {
            chapter
                .segments
                .iter()
                .filter(move |segment| !skip_translated || segment.translation.trim().is_empty())
                .map(move |segment| {
                    TranslationSegment::new(&segment.id, &chapter.title, &segment.source)
                })
        })
        .collect()
}

fn apply_translations(project: &mut BookProject, translations: &HashMap<String, String>) -> usize {
    let mut issues = 0;
    for segment in project
        .chapters
        .iter_mut()
        .flat_map(|chapter| &mut chapter.segments)
    {
        let Some(translation) = translations.get(&segment.id) else {
            continue;
        };
        segment.translation.clone_from(translation);
        let findings = check_translation(&segment.source, translation, None);
        if findings.is_empty() {
            segment.status = SegmentStatus::Draft;
            segment.qa_note = None;
        } else {
            issues += 1;
            segment.status = SegmentStatus::Issue;
            segment.qa_note = Some(
                findings
                    .iter()
                    .map(|finding| format!("{:?}", finding.code))
                    .collect::<Vec<_>>()
                    .join("、"),
            );
        }
    }
    issues
}

fn language_prompt(language: &LanguageInput) -> String {
    if language.code == "auto" {
        "Auto-detect the source language".to_owned()
    } else {
        format!("{} ({})", language.name, language.code)
    }
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(super) fn export_book_project(
    app: tauri::AppHandle,
    request: ExportBookInput,
) -> Result<BookExportRecord, String> {
    let ExportBookInput {
        project,
        format,
        profile,
    } = request;
    let extension = format.extension();
    let suggested_name = format!(
        "{}.{}.{}",
        safe_file_stem(&project.title),
        project.target_language,
        extension
    );
    let label = match format {
        BookExportFormat::Markdown => "Markdown 书稿",
        BookExportFormat::Docx => "Word 出版稿",
        BookExportFormat::Epub => "EPUB 3 电子书",
        BookExportFormat::Pdf => "PDF 印刷稿",
    };
    let path = rfd::FileDialog::new()
        .set_title("导出翻译书稿")
        .set_file_name(&suggested_name)
        .add_filter(label, &[extension])
        .save_file()
        .ok_or_else(|| "未选择导出位置".to_owned())?;
    match format {
        BookExportFormat::Markdown => export_markdown(&project, &path),
        BookExportFormat::Docx => export_docx(&project, &path),
        BookExportFormat::Epub => export_epub(&project, &path),
        BookExportFormat::Pdf => {
            let font = read_publication_font()?;
            export_pdf(&project, &path, &profile, &font)
        }
    }
    .map_err(|error| error.to_string())?;
    write_project(&books_directory(&app)?, &project)?;
    let exported_at_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX);
    let record = BookExportRecord {
        id: format!("{}-{exported_at_unix_ms}", project.id),
        project_id: project.id,
        book_title: project.title,
        format,
        output_path: path.to_string_lossy().into_owned(),
        target_language: project.target_language,
        exported_at_unix_ms,
        profile,
    };
    append_export_history(&export_history_path(&app)?, record.clone())?;
    Ok(record)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
pub(super) fn list_book_export_history(
    app: tauri::AppHandle,
    project_id: String,
) -> Result<Vec<BookExportRecord>, String> {
    read_export_history(&export_history_path(&app)?, Some(&project_id))
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)] // Tauri deserializes command arguments by value.
pub(super) fn open_book_export_location(path: String) -> Result<(), String> {
    let output = PathBuf::from(&path);
    if !output.exists() {
        return Err("导出文件已被移动或删除".to_owned());
    }
    std::process::Command::new("explorer.exe")
        .arg(format!("/select,{}", output.display()))
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("打开导出位置失败：{error}"))
}

fn safe_file_stem(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|character| match character {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => character,
        })
        .collect()
}

fn read_publication_font() -> Result<Vec<u8>, String> {
    [
        r"C:\Windows\Fonts\simfang.ttf",
        r"C:\Windows\Fonts\NotoSerifSC-VF.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ]
    .iter()
    .find_map(|path| fs::read(path).ok())
    .ok_or_else(|| "未找到可嵌入 PDF 的中文字体（Noto Serif SC、仿宋或宋体）".to_owned())
}

fn read_export_history(
    path: &Path,
    project_id: Option<&str>,
) -> Result<Vec<BookExportRecord>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(path).map_err(|error| format!("读取书籍导出历史失败：{error}"))?;
    let mut history: Vec<BookExportRecord> =
        serde_json::from_slice(&bytes).map_err(|error| format!("书籍导出历史损坏：{error}"))?;
    if let Some(project_id) = project_id {
        history.retain(|record| record.project_id == project_id);
    }
    history.sort_by(|left, right| right.exported_at_unix_ms.cmp(&left.exported_at_unix_ms));
    Ok(history)
}

fn append_export_history(path: &Path, record: BookExportRecord) -> Result<(), String> {
    let mut history = read_export_history(path, None)?;
    history.push(record);
    history.sort_by(|left, right| right.exported_at_unix_ms.cmp(&left.exported_at_unix_ms));
    if history.len() > 200 {
        history.truncate(200);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("创建导出历史目录失败：{error}"))?;
    }
    let json = serde_json::to_vec_pretty(&history)
        .map_err(|error| format!("序列化书籍导出历史失败：{error}"))?;
    fs::write(path, json).map_err(|error| format!("保存书籍导出历史失败：{error}"))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use game_translator_content_book::{
        parse_book, BookExportFormat, BookExportProfile, BookExportRecord,
    };

    #[test]
    fn persists_and_lists_book_projects() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("book.txt");
        fs::write(&source, "第一章 开始\n\nHello world.").unwrap();
        let project = parse_book(&source).unwrap();
        let state = directory.path().join("state");

        super::write_project(&state, &project).unwrap();
        let loaded = super::read_projects(&state).unwrap();

        assert_eq!(loaded, vec![project]);
    }

    #[test]
    fn persists_export_history_newest_first_and_filters_by_project() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("book-export-history.json");
        let first = export_record("first", "book-a", 10);
        let second = export_record("second", "book-a", 20);
        let unrelated = export_record("third", "book-b", 30);

        super::append_export_history(&path, first.clone()).unwrap();
        super::append_export_history(&path, second.clone()).unwrap();
        super::append_export_history(&path, unrelated).unwrap();

        assert_eq!(
            super::read_export_history(&path, Some("book-a")).unwrap(),
            vec![second, first]
        );
    }

    #[test]
    fn sanitizes_windows_export_file_names() {
        assert_eq!(
            super::safe_file_stem("书名：测试/终稿?"),
            "书名：测试_终稿_"
        );
    }

    #[test]
    fn full_book_translation_collects_all_chapters_but_skips_existing_translations() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("book.txt");
        fs::write(
            &source,
            "第一章 开始\n\nHello world.\n\n第二章 继续\n\nGoodbye world.",
        )
        .unwrap();
        let mut project = parse_book(&source).unwrap();
        project.chapters[0].segments[0].translation = "你好，世界。".to_owned();

        let segments = super::segments_for_translation(&project, None, true);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].source, "Goodbye world.");
    }

    fn export_record(id: &str, project_id: &str, exported_at_unix_ms: u64) -> BookExportRecord {
        BookExportRecord {
            id: id.to_owned(),
            project_id: project_id.to_owned(),
            book_title: "书名".to_owned(),
            format: BookExportFormat::Docx,
            output_path: format!(r"C:\exports\{id}.docx"),
            target_language: "zh-CN".to_owned(),
            exported_at_unix_ms,
            profile: BookExportProfile::default(),
        }
    }
}
