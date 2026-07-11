#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QaSeverity {
    Blocking,
    Warning,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QaCode {
    EmptyTranslation,
    UnchangedTranslation,
    LengthExceeded,
    LeakedPlaceholder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QaFinding {
    pub code: QaCode,
    pub severity: QaSeverity,
}

#[must_use]
pub fn check_translation(
    source: &str,
    target: &str,
    max_characters: Option<usize>,
) -> Vec<QaFinding> {
    let mut findings = Vec::new();

    if target.trim().is_empty() {
        findings.push(QaFinding {
            code: QaCode::EmptyTranslation,
            severity: QaSeverity::Warning,
        });
    } else if source == target {
        findings.push(QaFinding {
            code: QaCode::UnchangedTranslation,
            severity: QaSeverity::Warning,
        });
    }

    if max_characters.is_some_and(|maximum| target.chars().count() > maximum) {
        findings.push(QaFinding {
            code: QaCode::LengthExceeded,
            severity: QaSeverity::Warning,
        });
    }

    if target.contains("<ph") {
        findings.push(QaFinding {
            code: QaCode::LeakedPlaceholder,
            severity: QaSeverity::Blocking,
        });
    }

    findings
}
