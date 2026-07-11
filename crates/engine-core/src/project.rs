use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EngineKind {
    RpgMakerMv,
    RpgMakerMz,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DetectedProject {
    pub root: PathBuf,
    pub data_dir: PathBuf,
    pub engine: EngineKind,
}
