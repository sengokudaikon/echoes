//! Platform-specific notification handling

#[cfg(target_os = "macos")]
use mac_notification_sys::Notification;
#[cfg(target_os = "linux")]
use notify_rust::Notification as LinuxNotification;

#[cfg(target_os = "linux")]
use crate::PlatformError;
use crate::Result;

/// Sends a platform-specific notification with the given title and message.
///
/// # Errors
///
/// Returns an error if the notification system fails to send the notification.
/// On Linux, this can occur if the notification daemon is not running.
pub fn send_notification(title: &str, message: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let _ = Notification::new().title(title).subtitle(message).send();
        Ok(())
    }

    #[cfg(target_os = "windows")]
    {
        // TODO: Implement Windows notifications
        tracing::info!("Notification: {}: {}", title, message);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        LinuxNotification::new()
            .summary(title)
            .body(message)
            .show()
            .map_err(|e| PlatformError::SystemError(format!("Failed to send notification: {}", e)))?;
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        tracing::info!("Notification: {}: {}", title, message);
        Ok(())
    }
}

/// Sends an error notification with the given error message.
///
/// # Errors
///
/// Returns an error if the underlying notification system fails.
pub fn send_error_notification(error: &str) -> Result<()> {
    send_notification("Echoes Error", error)
}

/// Sends a success notification with the given message.
///
/// # Errors
///
/// Returns an error if the underlying notification system fails.
pub fn send_success_notification(message: &str) -> Result<()> {
    send_notification("Echoes", message)
}
