use game_translator_qa_core::{check_translation, QaCode, QaSeverity};

#[test]
fn reports_empty_and_unchanged_translations_as_warnings() {
    let empty = check_translation("Moonstone", "", None);
    let unchanged = check_translation("Moonstone", "Moonstone", None);

    assert_eq!(empty[0].code, QaCode::EmptyTranslation);
    assert_eq!(empty[0].severity, QaSeverity::Warning);
    assert_eq!(unchanged[0].code, QaCode::UnchangedTranslation);
}

#[test]
fn reports_excessive_length_as_a_warning() {
    let findings = check_translation("短い", "这是一段明显过长的中文翻译", Some(8));

    assert_eq!(findings[0].code, QaCode::LengthExceeded);
    assert_eq!(findings[0].severity, QaSeverity::Warning);
}

#[test]
fn leaked_placeholder_markup_blocks_export() {
    let findings = check_translation("\\V[1]", "变量 <ph id=\"0\"/>", None);

    assert_eq!(findings[0].code, QaCode::LeakedPlaceholder);
    assert_eq!(findings[0].severity, QaSeverity::Blocking);
}
