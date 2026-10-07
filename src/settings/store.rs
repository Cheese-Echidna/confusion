//! Local single-file settings persistence for toolbar pins, preserving other JSON fields.
//! Exports toolbar_pins/save_toolbar_pins; workspace loads IDs from its static catalog.
//! Future settings and keymap editors must share this file and atomic write path.
use std::{collections::HashSet, path::PathBuf};
pub(crate) fn path() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|root| root.join("confusion/settings.json"))
}
pub fn toolbar_pins() -> Option<HashSet<String>> {
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(path()?).ok()?).ok()?;
    serde_json::from_value(value.get("toolbar_pins")?.clone()).ok()
}
pub fn save_toolbar_pins(pins: impl IntoIterator<Item = impl AsRef<str>>) -> Result<(), String> {
    let path = path().ok_or("Cannot locate local settings directory")?;
    save_at(&path, pins)
}
fn save_at(
    path: &std::path::Path,
    pins: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<(), String> {
    use std::io::Write;
    let parent = path.parent().unwrap();
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut value = if path.exists() {
        serde_json::from_slice::<serde_json::Value>(
            &std::fs::read(path).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?
    } else {
        serde_json::json!({})
    };
    let object = value
        .as_object_mut()
        .ok_or("Settings must be a JSON object")?;
    let mut ids: Vec<String> = pins.into_iter().map(|p| p.as_ref().to_owned()).collect();
    ids.sort();
    object.insert("toolbar_pins".into(), serde_json::json!(ids));
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temporary
        .write_all(&serde_json::to_vec_pretty(&value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn pins_preserve_other_settings_and_reject_malformed_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, br#"{"keybindings":{"undo":"ctrl-z"},"future":7}"#).unwrap();
        super::save_at(&path, ["sketch-line", "solid-extrude"]).unwrap();
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(json["future"], 7);
        assert_eq!(json["keybindings"]["undo"], "ctrl-z");
        assert_eq!(
            json["toolbar_pins"],
            serde_json::json!(["sketch-line", "solid-extrude"])
        );
        std::fs::write(&path, "broken JSON").unwrap();
        assert!(super::save_at(&path, ["sketch-line"]).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "broken JSON");
    }
}
