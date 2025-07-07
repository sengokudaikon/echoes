//! Platform-specific permission handling

#[cfg(target_os = "macos")]
use core_foundation::base::TCFType;
#[cfg(target_os = "macos")]
use core_foundation::boolean::CFBoolean;
#[cfg(target_os = "macos")]
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
#[cfg(target_os = "macos")]
use core_foundation::string::CFString;

use crate::{PlatformError, Result};

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
}

#[cfg(target_os = "macos")]
pub fn check_accessibility_permissions(prompt: bool) -> bool {
    unsafe {
        let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
        let value = CFBoolean::from(prompt);

        let options = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);

        let is_trusted = AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef());

        tracing::debug!(
            "Accessibility permissions check: trusted={}, prompt={}",
            is_trusted,
            prompt
        );

        is_trusted
    }
}

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permissions(_prompt: bool) -> bool {
    true
}

pub fn ensure_permissions() -> Result<bool> {
    tracing::debug!("Checking system permissions");

    #[cfg(target_os = "macos")]
    {
        if check_accessibility_permissions(false) {
            tracing::debug!("Accessibility permissions already granted");
            return Ok(true);
        }

        tracing::debug!("Accessibility permissions not granted, prompting user");

        if check_accessibility_permissions(true) {
            tracing::debug!("User granted accessibility permissions");
            Ok(true)
        } else {
            tracing::error!("User denied accessibility permissions");
            Err(PlatformError::PermissionDenied(
                "Accessibility permissions required. Please grant access in System Settings > Privacy & Security > \
                 Accessibility, then restart the app."
                    .to_string(),
            ))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        tracing::debug!("Non-macOS platform, no special permissions needed");
        Ok(true)
    }
}

#[must_use]
pub fn get_required_permissions_description() -> String {
    #[cfg(target_os = "macos")]
    {
        "This application requires accessibility permissions to capture keyboard events globally. Please grant access \
         in System Settings > Privacy & Security > Accessibility."
            .to_string()
    }

    #[cfg(target_os = "windows")]
    {
        "This application may require administrator privileges for global keyboard capture on Windows.".to_string()
    }

    #[cfg(target_os = "linux")]
    {
        "This application may require your user to be in the 'input' group for keyboard capture on Linux.".to_string()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "Platform-specific permissions may be required for global keyboard capture.".to_string()
    }
}
