use game_translator_app_core::PerformanceSettings;

#[test]
fn translation_performance_policy_is_owned_by_the_application_layer() {
    assert_eq!(
        PerformanceSettings::for_provider("ollama", "fast"),
        PerformanceSettings::new(48, 20_000, 2, 2)
    );
    assert_eq!(
        PerformanceSettings::for_provider("openai", "balanced"),
        PerformanceSettings::new(24, 12_000, 8, 64)
    );
}
