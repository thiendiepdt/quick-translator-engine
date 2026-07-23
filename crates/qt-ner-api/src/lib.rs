//! Dedicated HTTP composition layer for the name-filter pipeline.

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use qt_core::{Dictionaries, DictionaryDefaults, Engine};

pub use qt_api::{AppState, NameFilterServices};

/// Build a name-filter state from already loaded dictionaries.
pub fn build_state(
    dictionaries: Dictionaries,
    dictionary_defaults: DictionaryDefaults,
) -> Arc<AppState> {
    Arc::new(AppState {
        engine: Arc::new(Engine::from_dicts(dictionaries)),
        dictionary_defaults: Arc::new(dictionary_defaults),
        name_filter_services: NameFilterServices::from_env(),
    })
}

/// Load QT-compatible dictionaries and initialize optional providers from the environment.
pub fn load_state(data_dir: &Path) -> std::io::Result<Arc<AppState>> {
    let (dictionaries, defaults) = Dictionaries::load_with_defaults(data_dir)?;
    Ok(build_state(dictionaries, defaults))
}

/// Expose only health, provider capabilities, and name filtering.
pub fn build_router(state: Arc<AppState>) -> Router {
    qt_api::build_name_filter_router(state)
}
