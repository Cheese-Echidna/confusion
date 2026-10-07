//! Platform-native derived runtime storage, separate from document files and settings.
use std::path::PathBuf;
pub fn recovery_directory() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("CONFUSION_RECOVERY_DIR") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var_os("HOME").map(PathBuf::from);
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base = home.map(|p| p.join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| home.map(|p| p.join(".local/state")));
    base.map(|p| p.join("confusion/recovery")).ok_or_else(|| {
        "Cannot locate recovery storage; set CONFUSION_RECOVERY_DIR to a writable directory".into()
    })
}
