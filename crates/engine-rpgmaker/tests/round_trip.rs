use std::{collections::HashMap, fs, path::Path, path::PathBuf};

use game_translator_engine_rpgmaker::{detect_project, write_translations};

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

#[test]
fn writes_only_targeted_strings_to_a_separate_directory() {
    let game = TempDirectory::new("round-trip-game");
    let output = TempDirectory::new("round-trip-output");
    copy_fixture(&game.0);
    let project = detect_project(&game.0).unwrap();
    let original_path = game.0.join("data/Map001.json");
    let original_bytes = fs::read(&original_path).unwrap();
    let translations = HashMap::from([(
        "Map001.json:events[1].pages[0].list[1]".to_string(),
        "终于到了。 \\V[1]".to_string(),
    )]);

    let written = write_translations(&project, &translations, &output.0).unwrap();

    assert_eq!(written, vec![output.0.join("data/Map001.json")]);
    assert_eq!(fs::read(original_path).unwrap(), original_bytes);
    let rendered: serde_json::Value =
        serde_json::from_slice(&fs::read(&written[0]).unwrap()).unwrap();
    assert_eq!(
        rendered["events"][1]["pages"][0]["list"][1]["parameters"][0],
        "终于到了。 \\V[1]"
    );
    assert_eq!(
        rendered["events"][1]["pages"][0]["list"][6]["parameters"][0],
        "$gameVariables.setValue(1, 12345);"
    );
}
