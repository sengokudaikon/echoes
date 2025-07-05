/// Manages system-level operations like permissions and platform-specific
/// features
pub struct SystemManager;

impl SystemManager {
    pub fn new() -> Self {
        Self
    }

    pub fn open_accessibility_settings() -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;

            // Open System Settings to Privacy & Security > Accessibility
            Command::new("open")
                .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                .spawn()
                .map_err(|e| format!("Failed to open System Settings: {e}"))?;
        }

        #[cfg(target_os = "windows")]
        {
            // Windows doesn't need special permissions for keyboard monitoring
            return Err("No special permissions needed on Windows".into());
        }

        #[cfg(target_os = "linux")]
        {
            // Linux might need the user to be in the input group
            return Err("On Linux, ensure you're in the 'input' group: sudo usermod -a -G input $USER".into());
        }

        Ok(())
    }
}

impl Default for SystemManager {
    fn default() -> Self {
        Self::new()
    }
}
