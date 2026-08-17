//! Shared application identity used by the daemon and desktop client.

#[cfg(debug_assertions)]
pub const APP_NAME: &str = "Anastasia Debug";
#[cfg(not(debug_assertions))]
pub const APP_NAME: &str = "Anastasia";

#[cfg(debug_assertions)]
pub const APP_ID: &str = "app.anastasia.debug";
#[cfg(not(debug_assertions))]
pub const APP_ID: &str = "app.anastasia";

#[cfg(debug_assertions)]
pub const DATA_DIRECTORY_NAME: &str = "Anastasia Debug";
#[cfg(not(debug_assertions))]
pub const DATA_DIRECTORY_NAME: &str = "Anastasia";
