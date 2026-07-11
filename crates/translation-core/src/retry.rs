use std::{thread, time::Duration};

use game_translator_provider_core::ProviderError;

const MAX_ATTEMPTS: usize = 2;
const INITIAL_DELAY: Duration = Duration::from_millis(25);

pub(crate) fn with_transient_retry<T>(
    mut operation: impl FnMut() -> Result<T, ProviderError>,
) -> Result<T, ProviderError> {
    for attempt in 0..MAX_ATTEMPTS {
        match operation() {
            Ok(value) => return Ok(value),
            Err(error) if is_transient(&error) && attempt + 1 < MAX_ATTEMPTS => {
                thread::sleep(INITIAL_DELAY * (1 << attempt));
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("retry loop always returns on its final attempt")
}

fn is_transient(error: &ProviderError) -> bool {
    matches!(
        error,
        ProviderError::RateLimited | ProviderError::Transport(_)
    )
}
