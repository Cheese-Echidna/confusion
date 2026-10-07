//! Context-aware Fusion defaults and validated overrides from the one settings file.
//! A null binding disables a default. Modeling shortcuts only resolve with viewport focus.
use std::collections::{BTreeMap, HashSet};
#[derive(Clone, Debug)]
pub struct Keymap {
    pub sketch: BTreeMap<String, String>,
    pub global: BTreeMap<String, String>,
}
impl Default for Keymap {
    fn default() -> Self {
        let sketch = [
            ("l", "sketch-line"),
            ("ctrl-a", "select-all"),
            ("m", "sketch-move-copy-sketch"),
            ("f", "sketch-sketch-fillet"),
            ("r", "sketch-rectangle"),
            ("c", "sketch-circle"),
            ("d", "sketch-dimension"),
            ("i", "sketch-measure-sketch"),
            ("t", "sketch-trim"),
            ("o", "sketch-offset"),
            ("x", "sketch-construction"),
            ("delete", "sketch-delete-geometry"),
            ("s", "command-search"),
        ]
        .into_iter()
        .map(|(k, c)| (k.into(), c.into()))
        .collect();
        let global = [
            ("ctrl-z", "undo"),
            ("ctrl-shift-z", "redo"),
            ("ctrl-y", "redo"),
            ("ctrl-s", "save"),
            ("ctrl-o", "open"),
            ("ctrl-n", "new"),
            ("escape", "cancel"),
            ("e", "solid-extrude"),
        ]
        .into_iter()
        .map(|(k, c)| (k.into(), c.into()))
        .collect();
        Self { sketch, global }
    }
}
impl Keymap {
    pub fn from_json(value: &serde_json::Value) -> Result<Self, String> {
        let mut result = Self::default();
        let Some(bindings) = value.get("keybindings") else {
            return Ok(result);
        };
        let scopes = bindings
            .as_object()
            .ok_or("keybindings must contain sketch/global objects")?;
        for (scope, overrides) in scopes {
            let map = match scope.as_str() {
                "sketch" => &mut result.sketch,
                "global" => &mut result.global,
                _ => return Err(format!("Unknown keybinding context {scope}")),
            };
            for (command, key) in overrides
                .as_object()
                .ok_or("Keybinding context must be an object")?
            {
                if !known(command) {
                    return Err(format!("Unknown shortcut command {command}"));
                }
                map.retain(|_, v| v != command);
                if key.is_null() {
                    continue;
                }
                let chord = key
                    .as_str()
                    .ok_or("Shortcut must be a chord string or null")?
                    .to_lowercase();
                if chord.is_empty() || chord.split('-').any(|part| part.is_empty()) {
                    return Err("Invalid shortcut chord".into());
                }
                if let Some(prior) = map.get(&chord) {
                    return Err(format!("Shortcut {chord} conflicts with {prior}"));
                }
                map.insert(chord, command.clone());
            }
        }
        Ok(result)
    }
    pub fn resolve(&self, chord: &str, sketch: bool, text_focus: bool) -> Option<&str> {
        if text_focus {
            return None;
        }
        if sketch && let Some(c) = self.sketch.get(chord) {
            return Some(c);
        }
        self.global.get(chord).map(String::as_str)
    }
}
fn known(id: &str) -> bool {
    let mut ids: HashSet<&str> = [
        "undo",
        "redo",
        "save",
        "open",
        "new",
        "cancel",
        "command-search",
        "select-all",
        "sketch-construction",
    ]
    .into_iter()
    .collect();
    for mode in [
        crate::ui::toolbar::Mode::Sketch,
        crate::ui::toolbar::Mode::Solid,
    ] {
        for group in crate::ui::toolbar::groups(mode) {
            ids.extend(group.features.iter().map(|f| f.id))
        }
    }
    ids.contains(id)
}
pub fn load() -> Result<Keymap, String> {
    let Some(path) = super::store::path() else {
        return Ok(Keymap::default());
    };
    if !path.exists() {
        return Ok(Keymap::default());
    }
    let json = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Keymap::from_json(&json)
}
