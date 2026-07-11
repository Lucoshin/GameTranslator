use std::path::PathBuf;

use game_translator_engine_core::{EngineError, EngineKind};
use game_translator_engine_rpgmaker::detect_project;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

#[test]
fn detects_rpg_maker_mv_by_its_www_data_directory() {
    let project = detect_project(&fixture("rpgmaker-mv-minimal")).unwrap();

    assert_eq!(project.engine, EngineKind::RpgMakerMv);
    assert_eq!(project.data_dir, fixture("rpgmaker-mv-minimal/www/data"));
}

#[test]
fn detects_rpg_maker_mz_by_its_root_data_directory() {
    let project = detect_project(&fixture("rpgmaker-mz-minimal")).unwrap();

    assert_eq!(project.engine, EngineKind::RpgMakerMz);
    assert_eq!(project.data_dir, fixture("rpgmaker-mz-minimal/data"));
}

#[test]
fn rejects_a_directory_without_rpg_maker_markers() {
    let error = detect_project(&fixture("unsupported")).unwrap_err();

    assert_eq!(error, EngineError::UnsupportedProject);
}

#[test]
fn reports_a_missing_system_database() {
    let error = detect_project(&fixture("rpgmaker-mz-missing-system")).unwrap_err();

    assert_eq!(
        error,
        EngineError::MissingRequiredFile(fixture("rpgmaker-mz-missing-system/data/System.json"))
    );
}
