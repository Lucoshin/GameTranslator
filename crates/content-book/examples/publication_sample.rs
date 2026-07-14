use std::{env, fs, path::PathBuf};

use game_translator_content_book::{
    export_docx, export_epub, export_pdf, parse_book, BookExportProfile,
};

fn main() {
    let output = env::args_os().nth(1).map_or_else(
        || env::temp_dir().join("game-translator-publication-sample"),
        PathBuf::from,
    );
    fs::create_dir_all(&output).unwrap();
    let source = output.join("sample-source.md");
    fs::write(
        &source,
        "# 第一章 潮声背后\n\n港口的雾，比往年更早抵达。\n\n黄昏时，朝圣者街沿途的灯火，已成了白色海洋中一座孤岛。\n\n纸页仍是冰凉的，尽管它整个下午都放在炉火旁。\n\n# 第二章 回信\n\n玛拉站在阁楼窗前，把那封尚未拆开的信贴在掌心。\n\n信封上没有回邮地址，只有那个她早已学会畏惧的小小墨记。",
    )
    .unwrap();
    let mut project = parse_book(&source).unwrap();
    "雾港来信".clone_into(&mut project.title);
    "示例作者".clone_into(&mut project.publication.author);
    "示例译者".clone_into(&mut project.publication.translator);
    "示例出版社".clone_into(&mut project.publication.publisher);
    "978-7-0000-0000-1".clone_into(&mut project.publication.isbn);
    "仅用于版式检查".clone_into(&mut project.publication.copyright);
    let font = [
        r"C:\Windows\Fonts\simfang.ttf",
        r"C:\Windows\Fonts\NotoSerifSC-VF.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ]
    .iter()
    .find_map(|path| fs::read(path).ok())
    .expect("需要可用的中文字体");

    export_docx(&project, &output.join("publication-sample.docx")).unwrap();
    export_epub(&project, &output.join("publication-sample.epub")).unwrap();
    export_pdf(
        &project,
        &output.join("publication-sample.pdf"),
        &BookExportProfile::default(),
        &font,
    )
    .unwrap();
}
