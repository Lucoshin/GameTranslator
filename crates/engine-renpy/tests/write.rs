use std::{collections::HashMap, path::PathBuf};

use game_translator_engine_renpy::{detect_project, language_identifier, write_translations};

#[test]
fn converts_bcp47_codes_to_valid_renpy_identifiers() {
    assert_eq!(language_identifier("ja-JP"), "ja_JP");
    assert_eq!(language_identifier("zh-Hans-CN"), "zh_Hans_CN");
    assert_eq!(language_identifier("123"), "_123");
}

#[test]
#[ignore = "requires an external Ren'Py distribution"]
fn writes_an_activation_script_for_the_normalized_language() {
    let root = std::env::var_os("GAME_TRANSLATOR_RENPY_FIXTURE")
        .map(PathBuf::from)
        .expect("set GAME_TRANSLATOR_RENPY_FIXTURE");
    let project = detect_project(&root).unwrap();
    let output = std::env::temp_dir().join(format!(
        "game-translator-renpy-language-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&output);

    let written = write_translations(&project, &HashMap::new(), &output, "ja-JP").unwrap();

    assert!(output.join("game/tl/ja_JP/script.rpy").is_file());
    let script = std::fs::read_to_string(output.join("game/tl/ja_JP/script.rpy")).unwrap();
    assert!(script.contains("translate ja_JP"));
    let activation =
        std::fs::read_to_string(output.join("game/game_translator_language.rpy")).unwrap();
    assert!(activation.contains("define config.language = \"ja_JP\""));
    assert!(written.contains(&output.join("game/game_translator_language.rpy")));
    let _ = std::fs::remove_dir_all(output);
}
