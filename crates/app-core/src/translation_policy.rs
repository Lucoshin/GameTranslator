#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PerformanceSettings {
    pub batch_size: usize,
    pub character_budget: usize,
    pub initial_concurrency: usize,
    pub max_concurrency: usize,
}

impl PerformanceSettings {
    #[must_use]
    pub const fn new(
        batch_size: usize,
        character_budget: usize,
        initial_concurrency: usize,
        max_concurrency: usize,
    ) -> Self {
        Self {
            batch_size,
            character_budget,
            initial_concurrency,
            max_concurrency,
        }
    }

    #[must_use]
    pub fn for_provider(kind: &str, mode: &str) -> Self {
        match (kind, mode) {
            ("ollama", "fast") => Self::new(48, 20_000, 2, 2),
            ("ollama", _) => Self::new(32, 16_000, 1, 1),
            (_, "stable") => Self::new(32, 16_000, 4, 16),
            (_, "fast") => Self::new(16, 8_000, 16, 128),
            _ => Self::new(24, 12_000, 8, 64),
        }
    }
}
