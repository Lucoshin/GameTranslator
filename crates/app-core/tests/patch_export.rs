use std::{collections::HashMap, fs, path::Path, path::PathBuf};

use game_translator_app_core::{PatchError, PatchPlan};
use game_translator_engine_rpgmaker::detect_project;
use game_translator_qa_core::{QaCode, QaFinding, QaSeverity};

struct TempDirectory(PathBuf);

impl TempDirectory {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("game-translator-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn copy_fixture(destination: &Path) {
    let source =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/rpgmaker-mz-dialogue");
    copy_directory(&source, destination);
}

fn copy_directory(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.path().is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn translation() -> HashMap<String, String> {
    HashMap::from([(
        "Map001.json:events[1].pages[0].list[1]".to_string(),
        "终于到了。 \\V[1]".to_string(),
    )])
}

#[test]
fn exports_hash_manifest_for_verified_files() {
    let game = TempDirectory::new("patch-game");
    let output = TempDirectory::new("patch-output");
    copy_fixture(&game.0);
    let project = detect_project(&game.0).unwrap();
    let plan = PatchPlan::capture(project).unwrap();

    let manifest = plan.export(&translation(), &[], &output.0).unwrap();

    assert_eq!(manifest.files.len(), 1);
    assert_eq!(
        manifest.files[0].relative_path,
        PathBuf::from("data/Map001.json")
    );
    assert_ne!(
        manifest.files[0].source_sha256,
        manifest.files[0].target_sha256
    );
    assert!(output.0.join("patch-manifest.json").is_file());
}

#[test]
fn blocks_export_when_qa_has_a_blocking_finding() {
    let game = TempDirectory::new("blocked-game");
    let output = TempDirectory::new("blocked-output");
    copy_fixture(&game.0);
    let project = detect_project(&game.0).unwrap();
    let plan = PatchPlan::capture(project).unwrap();
    let findings = [QaFinding {
        code: QaCode::LeakedPlaceholder,
        severity: QaSeverity::Blocking,
    }];

    assert_eq!(
        plan.export(&translation(), &findings, &output.0)
            .unwrap_err(),
        PatchError::BlockingQualityFindings
    );
}

#[test]
fn rejects_export_when_a_source_file_changed_after_capture() {
    let game = TempDirectory::new("changed-game");
    let output = TempDirectory::new("changed-output");
    copy_fixture(&game.0);
    let project = detect_project(&game.0).unwrap();
    let plan = PatchPlan::capture(project).unwrap();
    fs::write(game.0.join("data/Map001.json"), "{}").unwrap();

    assert!(matches!(
        plan.export(&translation(), &[], &output.0),
        Err(PatchError::SourceChanged(_))
    ));
}
