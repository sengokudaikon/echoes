#[cfg(target_os = "macos")]
use core_foundation::base::TCFType;
#[cfg(target_os = "macos")]
use core_foundation::boolean::CFBoolean;
#[cfg(target_os = "macos")]
use core_foundation::dictionary::{CFDictionary, CFDictionaryRef};
#[cfg(target_os = "macos")]
use core_foundation::string::CFString;

use crate::{log_debug, log_error};

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: CFDictionaryRef) -> bool;
}

#[cfg(target_os = "macos")]
pub fn check_accessibility_permissions(prompt: bool) -> bool {
    unsafe {
        let key = CFString::from_static_string("AXTrustedCheckOptionPrompt");
        let value = CFBoolean::from(prompt);
        
        let options = CFDictionary::from_CFType_pairs(&[
            (key.as_CFType(), value.as_CFType())
        ]);
        
        let is_trusted = AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef());
        
        log_debug!("Accessibility permissions check: trusted={}, prompt={}", is_trusted, prompt);
        
        is_trusted
    }
}

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permissions(_prompt: bool) -> bool {
    // On non-macOS platforms, we don't need special permissions
    true
}

pub fn ensure_permissions() -> Result<bool, String> {
    log_debug!("Checking system permissions");
    
    #[cfg(target_os = "macos")]
    {
        // First check without prompting
        if check_accessibility_permissions(false) {
            log_debug!("Accessibility permissions already granted");
            return Ok(true);
        }
        
        log_debug!("Accessibility permissions not granted, prompting user");
        
        // Check again with prompt
        if check_accessibility_permissions(true) {
            log_debug!("User granted accessibility permissions");
            return Ok(true);
        } else {
            log_error!("User denied accessibility permissions");
            return Err("Accessibility permissions required. Please grant access in System Settings > Privacy & Security > Accessibility, then restart the app.".to_string());
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        log_debug!("Non-macOS platform, no special permissions needed");
        Ok(true)
    }
}