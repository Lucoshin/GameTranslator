mod detect;
mod generate;
mod template;
mod write;

pub use detect::detect_project;
pub use generate::generate_templates;
pub use template::{extract_project, extract_templates};
pub use write::{language_identifier, write_translations};

pub struct RenPyAdapter;

impl game_translator_engine_core::EngineAdapter for RenPyAdapter {
    fn detect(
        &self,
        root: &std::path::Path,
    ) -> Result<
        game_translator_engine_core::DetectedProject,
        game_translator_engine_core::EngineError,
    > {
        detect_project(root)
    }

    fn extract(
        &self,
        project: &game_translator_engine_core::DetectedProject,
    ) -> Result<Vec<game_translator_engine_core::Segment>, game_translator_engine_core::EngineError>
    {
        extract_project(project)
    }
}
