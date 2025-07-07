//! Shortcut conflict detection system

use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use crate::shortcuts::{KeyCode, RecordingShortcut};

/// Severity level for shortcut conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictSeverity {
    Error,
    Warning,
    Info,
}

/// Information about a shortcut conflict
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictInfo {
    pub severity: ConflictSeverity,
    pub description: String,
    pub suggestion: Option<String>,
}

/// Pattern for matching shortcuts in lookup tables
#[derive(Hash, Eq, PartialEq, Clone)]
struct ShortcutPattern {
    key: KeyCode,
    modifiers: Vec<KeyCode>,
    platform: Option<&'static str>,
}

/// Trait for conflict detection strategies
trait ConflictDetector: Send + Sync {
    /// Check if the given shortcut conflicts with this detector's domain
    fn check(&self, shortcut: &RecordingShortcut) -> Option<ConflictInfo>;
    /// Get the name of this detector for debugging
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}

/// System shortcut conflict detector that checks against OS-level shortcuts
struct SystemConflictDetector {
    shortcuts: &'static HashMap<ShortcutPattern, &'static str>,
}

/// Application shortcut conflict detector that checks against common app
/// shortcuts
struct ApplicationConflictDetector {
    shortcuts: &'static HashMap<ShortcutPattern, ConflictInfo>,
}

/// Accessibility concern detector that checks for usability issues
struct AccessibilityDetector;

/// Cache for conflict detection results to improve performance
#[derive(Default)]
struct ConflictCache {
    cache: HashMap<RecordingShortcut, Vec<ConflictInfo>>,
}

/// Main conflict detection system that coordinates multiple detectors
pub struct ConflictDetectionSystem {
    detectors: Vec<Box<dyn ConflictDetector>>,
    cache: ConflictCache,
}

// Lookup tables for system shortcuts
static MACOS_SYSTEM_SHORTCUTS: LazyLock<HashMap<ShortcutPattern, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // Cmd+key shortcuts
    let cmd_shortcuts = [
        (KeyCode::Q, "Cmd+Q quits applications"),
        (KeyCode::W, "Cmd+W closes windows"),
        (KeyCode::H, "Cmd+H hides applications"),
        (KeyCode::M, "Cmd+M minimizes windows"),
        (KeyCode::Tab, "Cmd+Tab switches applications"),
        (KeyCode::Space, "Cmd+Space opens Spotlight search"),
    ];

    for (key, desc) in cmd_shortcuts {
        map.insert(
            ShortcutPattern {
                key,
                modifiers: vec![KeyCode::MetaLeft],
                platform: Some("macos"),
            },
            desc,
        );
    }

    map
});

#[allow(dead_code)]
static WINDOWS_SYSTEM_SHORTCUTS: LazyLock<HashMap<ShortcutPattern, &'static str>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    // Win+key shortcuts
    let win_shortcuts = [
        (KeyCode::L, "Win+L locks the computer"),
        (KeyCode::D, "Win+D shows desktop"),
        (KeyCode::Tab, "Win+Tab opens Task View"),
    ];

    for (key, desc) in win_shortcuts {
        map.insert(
            ShortcutPattern {
                key,
                modifiers: vec![KeyCode::MetaLeft],
                platform: Some("windows"),
            },
            desc,
        );
    }

    // Alt+Tab
    map.insert(
        ShortcutPattern {
            key: KeyCode::Tab,
            modifiers: vec![KeyCode::Alt],
            platform: Some("windows"),
        },
        "Alt+Tab switches windows",
    );

    map
});

static APPLICATION_SHORTCUTS: LazyLock<HashMap<ShortcutPattern, ConflictInfo>> = LazyLock::new(|| {
    let mut map = HashMap::new();

    let common_shortcuts = [
        (KeyCode::S, "Save", "most applications"),
        (KeyCode::C, "Copy", "most applications"),
        (KeyCode::V, "Paste", "most applications"),
        (KeyCode::X, "Cut", "most applications"),
        (KeyCode::Z, "Undo", "most applications"),
        (KeyCode::A, "Select All", "most applications"),
        (KeyCode::F, "Find", "most applications"),
        (KeyCode::N, "New", "most applications"),
        (KeyCode::O, "Open", "most applications"),
        (KeyCode::R, "Refresh/Reload", "most applications"),
    ];

    for (key, action, context) in common_shortcuts {
        let conflict_info = ConflictInfo {
            severity: ConflictSeverity::Warning,
            description: format!("Conflicts with {action} in {context}"),
            suggestion: Some(format!("This will prevent {action} while recording")),
        };

        // Both Ctrl and Cmd variations
        for modifier in [KeyCode::ControlLeft, KeyCode::MetaLeft] {
            map.insert(
                ShortcutPattern {
                    key,
                    modifiers: vec![modifier],
                    platform: None,
                },
                conflict_info.clone(),
            );
        }
    }

    map
});

impl SystemConflictDetector {
    fn new() -> Self {
        #[cfg(target_os = "macos")]
        let shortcuts = &*MACOS_SYSTEM_SHORTCUTS;
        #[cfg(target_os = "windows")]
        let shortcuts = &*WINDOWS_SYSTEM_SHORTCUTS;
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let shortcuts = {
            static EMPTY: LazyLock<HashMap<ShortcutPattern, &'static str>> = LazyLock::new(HashMap::new);
            &*EMPTY
        };

        Self { shortcuts }
    }
}

impl ConflictDetector for SystemConflictDetector {
    fn check(&self, shortcut: &RecordingShortcut) -> Option<ConflictInfo> {
        let pattern = ShortcutPattern {
            key: shortcut.key,
            modifiers: shortcut.modifiers.clone(),
            platform: None,
        };

        self.shortcuts.get(&pattern).map(|desc| ConflictInfo {
            severity: ConflictSeverity::Error,
            description: (*desc).into(),
            suggestion: Some("System shortcuts cannot be overridden".into()),
        })
    }

    fn name(&self) -> &'static str {
        "System"
    }
}

impl ApplicationConflictDetector {
    fn new() -> Self {
        Self {
            shortcuts: &*APPLICATION_SHORTCUTS,
        }
    }
}

impl ConflictDetector for ApplicationConflictDetector {
    fn check(&self, shortcut: &RecordingShortcut) -> Option<ConflictInfo> {
        let pattern = ShortcutPattern {
            key: shortcut.key,
            modifiers: shortcut.modifiers.clone(),
            platform: None,
        };

        self.shortcuts.get(&pattern).cloned()
    }

    fn name(&self) -> &'static str {
        "Application"
    }
}

impl ConflictDetector for AccessibilityDetector {
    fn check(&self, shortcut: &RecordingShortcut) -> Option<ConflictInfo> {
        check_accessibility_concerns(shortcut)
    }

    fn name(&self) -> &'static str {
        "Accessibility"
    }
}

impl Default for ConflictDetectionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictDetectionSystem {
    /// Create a new conflict detection system with all detectors
    #[must_use]
    pub fn new() -> Self {
        let detectors: Vec<Box<dyn ConflictDetector>> = vec![
            Box::new(SystemConflictDetector::new()),
            Box::new(ApplicationConflictDetector::new()),
            Box::new(AccessibilityDetector),
        ];

        Self {
            detectors,
            cache: ConflictCache::default(),
        }
    }

    /// Check for conflicts with caching for performance
    pub fn check_conflicts(&mut self, shortcut: &RecordingShortcut) -> Vec<ConflictInfo> {
        // Check cache first
        if let Some(cached) = self.cache.cache.get(shortcut) {
            return cached.clone();
        }

        // Run all detectors
        let mut conflicts = Vec::new();
        for detector in &self.detectors {
            if let Some(conflict) = detector.check(shortcut) {
                conflicts.push(conflict);
            }
        }

        // Cache the result
        self.cache.cache.insert(shortcut.clone(), conflicts.clone());

        // Limit cache size to prevent memory growth
        if self.cache.cache.len() > 1000 {
            self.cache.cache.clear();
        }

        conflicts
    }

    /// Clear the conflict detection cache
    pub fn clear_cache(&mut self) {
        self.cache.cache.clear();
    }
}

// Global instance with thread-safe access
static CONFLICT_SYSTEM: LazyLock<Mutex<ConflictDetectionSystem>> =
    LazyLock::new(|| Mutex::new(ConflictDetectionSystem::new()));

/// Main entry point for checking shortcut conflicts
pub fn check_shortcut_conflicts(shortcut: &RecordingShortcut) -> Vec<ConflictInfo> {
    CONFLICT_SYSTEM
        .lock()
        .map_or_else(|_| Vec::new(), |mut system| system.check_conflicts(shortcut))
}

/// Check for accessibility concerns with a shortcut
fn check_accessibility_concerns(shortcut: &RecordingShortcut) -> Option<ConflictInfo> {
    // Check if shortcut is difficult to press with one hand
    if !is_easily_accessible(shortcut) {
        return Some(ConflictInfo {
            severity: ConflictSeverity::Info,
            description: "This combination might be difficult to press with one hand".into(),
            suggestion: Some("Consider using keys closer together or fewer modifiers".into()),
        });
    }

    // Check for modifier-heavy shortcuts
    if shortcut.modifiers.len() >= 3 {
        return Some(ConflictInfo {
            severity: ConflictSeverity::Info,
            description: "Many modifier keys may be hard to press simultaneously".into(),
            suggestion: Some("Consider using fewer modifiers for easier access".into()),
        });
    }

    None
}

/// Check if a shortcut is easily accessible with one hand
fn is_easily_accessible(shortcut: &RecordingShortcut) -> bool {
    // Check if the combination can be pressed comfortably with one hand
    let left_side_keys = [
        KeyCode::Q,
        KeyCode::W,
        KeyCode::E,
        KeyCode::R,
        KeyCode::T,
        KeyCode::A,
        KeyCode::S,
        KeyCode::D,
        KeyCode::F,
        KeyCode::G,
        KeyCode::Z,
        KeyCode::X,
        KeyCode::C,
        KeyCode::V,
        KeyCode::B,
        KeyCode::Tab,
        KeyCode::CapsLock,
        KeyCode::ShiftLeft,
        KeyCode::ControlLeft,
        KeyCode::Num1,
        KeyCode::Num2,
        KeyCode::Num3,
        KeyCode::Num4,
        KeyCode::Num5,
    ];

    let right_side_keys = [
        KeyCode::Y,
        KeyCode::U,
        KeyCode::I,
        KeyCode::O,
        KeyCode::P,
        KeyCode::H,
        KeyCode::J,
        KeyCode::K,
        KeyCode::L,
        KeyCode::N,
        KeyCode::M,
        KeyCode::ShiftRight,
        KeyCode::ControlRight,
        KeyCode::Num6,
        KeyCode::Num7,
        KeyCode::Num8,
        KeyCode::Num9,
        KeyCode::Num0,
    ];

    let main_key_left = left_side_keys.contains(&shortcut.key);
    let main_key_right = right_side_keys.contains(&shortcut.key);

    // If main key is on one side, check if all modifiers are also on the same side
    if main_key_left {
        shortcut
            .modifiers
            .iter()
            .all(|m| left_side_keys.contains(m) || is_universal_modifier(*m))
    } else if main_key_right {
        shortcut
            .modifiers
            .iter()
            .all(|m| right_side_keys.contains(m) || is_universal_modifier(*m))
    } else {
        // Main key is in the middle (like Space), generally accessible
        true
    }
}

/// Check if a key is a universal modifier (accessible from both sides)
const fn is_universal_modifier(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::Alt | KeyCode::AltGr | KeyCode::MetaLeft | KeyCode::MetaRight
    )
}
