use crate::descriptor::CommandId;
use bevy::input::keyboard::KeyCode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EditorKey {
    KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ,
    KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT,
    KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
    Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
    Space, Enter, Escape, Delete, Backspace, Tab,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Other,
}

impl From<KeyCode> for EditorKey {
    fn from(code: KeyCode) -> Self {
        match code {
            KeyCode::KeyA => Self::KeyA,
            KeyCode::KeyB => Self::KeyB,
            KeyCode::KeyC => Self::KeyC,
            KeyCode::KeyD => Self::KeyD,
            KeyCode::KeyE => Self::KeyE,
            KeyCode::KeyF => Self::KeyF,
            KeyCode::KeyG => Self::KeyG,
            KeyCode::KeyH => Self::KeyH,
            KeyCode::KeyI => Self::KeyI,
            KeyCode::KeyJ => Self::KeyJ,
            KeyCode::KeyK => Self::KeyK,
            KeyCode::KeyL => Self::KeyL,
            KeyCode::KeyM => Self::KeyM,
            KeyCode::KeyN => Self::KeyN,
            KeyCode::KeyO => Self::KeyO,
            KeyCode::KeyP => Self::KeyP,
            KeyCode::KeyQ => Self::KeyQ,
            KeyCode::KeyR => Self::KeyR,
            KeyCode::KeyS => Self::KeyS,
            KeyCode::KeyT => Self::KeyT,
            KeyCode::KeyU => Self::KeyU,
            KeyCode::KeyV => Self::KeyV,
            KeyCode::KeyW => Self::KeyW,
            KeyCode::KeyX => Self::KeyX,
            KeyCode::KeyY => Self::KeyY,
            KeyCode::KeyZ => Self::KeyZ,
            KeyCode::Digit0 => Self::Digit0,
            KeyCode::Digit1 => Self::Digit1,
            KeyCode::Digit2 => Self::Digit2,
            KeyCode::Digit3 => Self::Digit3,
            KeyCode::Digit4 => Self::Digit4,
            KeyCode::Digit5 => Self::Digit5,
            KeyCode::Digit6 => Self::Digit6,
            KeyCode::Digit7 => Self::Digit7,
            KeyCode::Digit8 => Self::Digit8,
            KeyCode::Digit9 => Self::Digit9,
            KeyCode::Space => Self::Space,
            KeyCode::Enter => Self::Enter,
            KeyCode::Escape => Self::Escape,
            KeyCode::Delete => Self::Delete,
            KeyCode::Backspace => Self::Backspace,
            KeyCode::Tab => Self::Tab,
            KeyCode::F1 => Self::F1,
            KeyCode::F2 => Self::F2,
            KeyCode::F3 => Self::F3,
            KeyCode::F4 => Self::F4,
            KeyCode::F5 => Self::F5,
            KeyCode::F6 => Self::F6,
            KeyCode::F7 => Self::F7,
            KeyCode::F8 => Self::F8,
            KeyCode::F9 => Self::F9,
            KeyCode::F10 => Self::F10,
            KeyCode::F11 => Self::F11,
            KeyCode::F12 => Self::F12,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyChord {
    pub key: EditorKey,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyChord {
    pub const fn new(key: EditorKey) -> Self {
        Self {
            key,
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    pub fn from_keycode(code: KeyCode) -> Self {
        Self::new(EditorKey::from(code))
    }

    pub const fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub const fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub const fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn to_shortcut_string(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        let key_str = match self.key {
            EditorKey::Space => "Space",
            EditorKey::Escape => "Esc",
            EditorKey::Delete => "Delete",
            EditorKey::Enter => "Enter",
            EditorKey::Backspace => "Backspace",
            EditorKey::Tab => "Tab",
            _ => "Key",
        };
        parts.push(key_str);
        parts.join("+")
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct KeybindingMap {
    bindings: HashMap<KeyChord, CommandId>,
}

impl KeybindingMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_defaults() -> Self {
        let mut map = Self::new();
        map.bind(KeyChord::new(EditorKey::KeyZ).ctrl(), CommandId::from("edit.undo"));
        map.bind(KeyChord::new(EditorKey::KeyY).ctrl(), CommandId::from("edit.redo"));
        map.bind(KeyChord::new(EditorKey::KeyZ).ctrl().shift(), CommandId::from("edit.redo"));
        map.bind(KeyChord::new(EditorKey::Delete), CommandId::from("graph.delete_selected"));
        map.bind(KeyChord::new(EditorKey::KeyP).ctrl(), CommandId::from("app.command_palette"));
        map.bind(KeyChord::new(EditorKey::Space), CommandId::from("graph.quick_palette"));
        map.bind(KeyChord::new(EditorKey::KeyF), CommandId::from("view.zoom_fit"));
        map.bind(KeyChord::new(EditorKey::Escape), CommandId::from("view.exit_focus"));
        map
    }

    pub fn bind(&mut self, chord: KeyChord, command_id: impl Into<CommandId>) {
        self.bindings.insert(chord, command_id.into());
    }

    pub fn unbind(&mut self, chord: &KeyChord) -> Option<CommandId> {
        self.bindings.remove(chord)
    }

    pub fn get_command(&self, chord: &KeyChord) -> Option<&CommandId> {
        self.bindings.get(chord)
    }

    pub fn get_chord_for_command(&self, command_id: &CommandId) -> Option<KeyChord> {
        self.bindings
            .iter()
            .find_map(|(chord, id)| if id == command_id { Some(*chord) } else { None })
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let serialized = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())?;
        std::fs::write(path, serialized).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        ron::from_str(&content).map_err(|e| e.to_string())
    }
}
