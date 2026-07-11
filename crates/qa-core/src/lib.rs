mod checks;
mod placeholders;

pub use checks::{check_translation, QaCode, QaFinding, QaSeverity};
pub use placeholders::{
    protect_placeholders, restore_placeholders, validate_control_codes, PlaceholderError,
    ProtectedText,
};
