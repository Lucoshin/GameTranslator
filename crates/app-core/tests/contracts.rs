use game_translator_app_core::{AppError, AppErrorCode, TaskStatus};

#[test]
fn application_errors_expose_stable_codes_without_losing_the_message() {
    let error = AppError::new(AppErrorCode::ProviderRateLimited, "模型服务限流");

    assert_eq!(error.code(), AppErrorCode::ProviderRateLimited);
    assert_eq!(error.message(), "模型服务限流");
    assert_eq!(error.to_string(), "模型服务限流");
}

#[test]
fn task_status_only_allows_documented_transitions() {
    assert!(TaskStatus::Pending.can_transition_to(TaskStatus::Running));
    assert!(TaskStatus::Running.can_transition_to(TaskStatus::Paused));
    assert!(TaskStatus::Paused.can_transition_to(TaskStatus::Running));
    assert!(TaskStatus::Running.can_transition_to(TaskStatus::Completed));
    assert!(!TaskStatus::Completed.can_transition_to(TaskStatus::Running));
    assert!(!TaskStatus::Failed.can_transition_to(TaskStatus::Completed));
}
