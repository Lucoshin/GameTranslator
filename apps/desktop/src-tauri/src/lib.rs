#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Starts the desktop application event loop.
///
/// # Panics
///
/// Panics when Tauri cannot initialize or run the platform event loop.
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to run GameTranslator");
}

#[cfg(test)]
mod tests {
    #[test]
    fn desktop_uses_the_core_product_name() {
        assert_eq!(game_translator_app_core::product_name(), "GameTranslator");
    }
}
