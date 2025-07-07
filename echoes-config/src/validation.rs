//! Shortcut validation logic

use std::collections::HashSet;

use crate::shortcuts::{is_modifier_key, normalize_modifier, RecordingShortcut};

/// Validation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyShortcut,
    ModifierOnly,
    DuplicateModifiers,
    SystemConflict(String),
    ConflictsDetected(Vec<crate::conflict::ConflictInfo>),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyShortcut => write!(f, "Shortcut cannot be empty"),
            Self::ModifierOnly => write!(f, "Shortcut cannot be only a modifier key"),
            Self::DuplicateModifiers => write!(f, "Duplicate modifier keys detected"),
            Self::SystemConflict(desc) => {
                write!(f, "Conflicts with system shortcut: {desc}")
            }
            Self::ConflictsDetected(conflicts) => {
                write!(f, "Conflicts detected: ")?;
                for (i, conflict) in conflicts.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", conflict.description)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate a recording shortcut
///
/// # Errors
///
/// Returns an error if the shortcut is invalid:
/// - `ModifierOnly`: If the main key is a modifier key but other modifiers are
///   also present
/// - `InvalidKey`: If the key is not supported or recognized
pub fn validate_shortcut(shortcut: &RecordingShortcut) -> Result<(), ValidationError> {
    // Allow single modifier keys as shortcuts (like Ctrl for recording)
    // Only reject if we have modifiers but the main key is also a modifier
    if is_modifier_key(&shortcut.key) && !shortcut.modifiers.is_empty() {
        return Err(ValidationError::ModifierOnly);
    }

    // Check for duplicate modifiers
    let mut seen_modifiers = HashSet::new();
    for modifier in &shortcut.modifiers {
        let normalized = normalize_modifier(modifier);
        if !seen_modifiers.insert(normalized) {
            return Err(ValidationError::DuplicateModifiers);
        }
    }

    // Check for system conflicts
    if let Some(conflict) = check_system_conflict(shortcut) {
        return Err(ValidationError::SystemConflict(conflict));
    }

    Ok(())
}

/// Check for system shortcut conflicts
fn check_system_conflict(shortcut: &RecordingShortcut) -> Option<String> {
    // Platform-specific system shortcut checking
    #[cfg(target_os = "macos")]
    {
        // Check common macOS system shortcuts
        let has_cmd = shortcut.modifiers.iter().any(|k| {
            matches!(
                k,
                crate::shortcuts::KeyCode::MetaLeft | crate::shortcuts::KeyCode::MetaRight
            )
        });

        if has_cmd {
            match &shortcut.key {
                crate::shortcuts::KeyCode::Q => return Some("Cmd+Q quits applications".into()),
                crate::shortcuts::KeyCode::W => return Some("Cmd+W closes windows".into()),
                crate::shortcuts::KeyCode::H => return Some("Cmd+H hides applications".into()),
                crate::shortcuts::KeyCode::M => return Some("Cmd+M minimizes windows".into()),
                crate::shortcuts::KeyCode::Tab => return Some("Cmd+Tab switches applications".into()),
                crate::shortcuts::KeyCode::Space => {
                    if shortcut.modifiers.len() == 1 {
                        return Some("Cmd+Space opens Spotlight search".into());
                    }
                }
                _ => {}
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Check common Windows system shortcuts
        let has_win = shortcut.modifiers.iter().any(|k| {
            matches!(
                k,
                crate::shortcuts::KeyCode::MetaLeft | crate::shortcuts::KeyCode::MetaRight
            )
        });
        let has_alt = shortcut
            .modifiers
            .iter()
            .any(|k| matches!(k, crate::shortcuts::KeyCode::Alt | crate::shortcuts::KeyCode::AltGr));

        if has_win {
            match &shortcut.key {
                crate::shortcuts::KeyCode::L => return Some("Win+L locks the computer".into()),
                crate::shortcuts::KeyCode::D => return Some("Win+D shows desktop".into()),
                crate::shortcuts::KeyCode::Tab => return Some("Win+Tab opens Task View".into()),
                _ => {}
            }
        }

        if has_alt && shortcut.key == crate::shortcuts::KeyCode::Tab {
            return Some("Alt+Tab switches windows".into());
        }
    }

    // Cross-platform shortcuts
    let has_ctrl = shortcut.modifiers.iter().any(|k| {
        matches!(
            k,
            crate::shortcuts::KeyCode::ControlLeft | crate::shortcuts::KeyCode::ControlRight
        )
    });
    let has_alt = shortcut
        .modifiers
        .iter()
        .any(|k| matches!(k, crate::shortcuts::KeyCode::Alt | crate::shortcuts::KeyCode::AltGr));

    if has_ctrl && has_alt && shortcut.key == crate::shortcuts::KeyCode::Delete {
        return Some("Ctrl+Alt+Delete is a system shortcut".into());
    }

    None
}
