use game_translator_qa_core::{protect_placeholders, restore_placeholders, PlaceholderError};

#[test]
fn protects_and_restores_rpg_maker_control_codes() {
    let protected = protect_placeholders("HP \\V[12]、角色 \\N[3]、颜色 \\C[4]、图标 \\I[8]");

    assert_eq!(
        protected.text,
        "HP <ph id=\"0\"/>、角色 <ph id=\"1\"/>、颜色 <ph id=\"2\"/>、图标 <ph id=\"3\"/>"
    );
    assert_eq!(
        restore_placeholders(
            &protected,
            "生命 <ph id=\"0\"/>、人物 <ph id=\"1\"/>、颜色 <ph id=\"2\"/>、图标 <ph id=\"3\"/>"
        )
        .unwrap(),
        "生命 \\V[12]、人物 \\N[3]、颜色 \\C[4]、图标 \\I[8]"
    );
}

#[test]
fn rejects_a_missing_placeholder() {
    let protected = protect_placeholders("\\V[1] / \\V[2]");

    assert_eq!(
        restore_placeholders(&protected, "<ph id=\"0\"/>").unwrap_err(),
        PlaceholderError::CountMismatch {
            expected: 2,
            actual: 1,
        }
    );
}

#[test]
fn rejects_an_added_placeholder() {
    let protected = protect_placeholders("\\V[1]");

    assert_eq!(
        restore_placeholders(&protected, "<ph id=\"0\"/> <ph id=\"1\"/>").unwrap_err(),
        PlaceholderError::CountMismatch {
            expected: 1,
            actual: 2,
        }
    );
}

#[test]
fn rejects_reordered_placeholders() {
    let protected = protect_placeholders("\\C[1]A\\C[0]");

    assert_eq!(
        restore_placeholders(&protected, "<ph id=\"1\"/>A<ph id=\"0\"/>").unwrap_err(),
        PlaceholderError::OrderMismatch {
            expected: vec![0, 1],
            actual: vec![1, 0],
        }
    );
}
