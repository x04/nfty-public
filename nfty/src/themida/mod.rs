#[cfg(feature = "themida")]
#[cfg(target_os = "windows")]
include!("SecureEngineSDK64.rs");

#[cfg(feature = "themida")]
#[cfg(target_os = "windows")]
include!("SecureEngineSDK64_CustomVMs.rs");
