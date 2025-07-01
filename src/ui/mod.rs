use eframe::egui;
use crate::config::{Config, SttProvider, RecordingShortcut, ShortcutMode, KeyCode};
use std::sync::mpsc;
use crate::keyboard::{KeyboardListener, KeyboardEvent};
use crate::{log_debug, permissions};

pub struct WhispoApp {
    config: Config,
    keyboard_rx: Option<mpsc::Receiver<KeyboardEvent>>,
    recording: bool,
    logs: Vec<String>,
    permissions_granted: bool,
    error_message: Option<String>,
}

impl WhispoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        let mut app = Self {
            config,
            keyboard_rx: None,
            recording: false,
            logs: vec!["App started".to_string()],
            permissions_granted: false,
            error_message: None,
        };
        
        // Check permissions and start keyboard listener
        app.init_keyboard_listener();
        
        app
    }
    
    fn init_keyboard_listener(&mut self) {
        match permissions::ensure_permissions() {
            Ok(true) => {
                self.permissions_granted = true;
                self.add_log("Permissions granted".to_string());
                
                // Set up keyboard listener
                let (tx, rx) = mpsc::channel();
                let listener = KeyboardListener::new(tx, self.config.recording_shortcut.clone());
                
                if let Err(e) = listener.start_listening() {
                    self.error_message = Some(format!("Failed to start keyboard listener: {}", e));
                    self.add_log(format!("Failed to start keyboard listener: {}", e));
                } else {
                    self.keyboard_rx = Some(rx);
                    self.add_log("Keyboard listener started".to_string());
                }
            }
            Ok(false) => {
                self.permissions_granted = false;
                self.error_message = Some("Permissions not granted".to_string());
                self.add_log("Permissions not granted".to_string());
            }
            Err(e) => {
                self.permissions_granted = false;
                self.error_message = Some(e.clone());
                self.add_log(format!("Permission error: {}", e));
            }
        }
    }
    
    fn add_log(&mut self, msg: String) {
        log_debug!("{}", msg);
        self.logs.push(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
        // Keep only last 100 logs
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
    
    fn open_accessibility_settings(&mut self) {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            
            // Open System Settings to Privacy & Security > Accessibility
            let result = Command::new("open")
                .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                .spawn();
            
            match result {
                Ok(_) => self.add_log("Opened System Settings".to_string()),
                Err(e) => self.add_log(format!("Failed to open System Settings: {}", e)),
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            // Windows doesn't need special permissions for keyboard monitoring
            self.add_log("No special permissions needed on Windows".to_string());
        }
        
        #[cfg(target_os = "linux")]
        {
            // Linux might need the user to be in the input group
            self.add_log("On Linux, ensure you're in the 'input' group: sudo usermod -a -G input $USER".to_string());
        }
    }
}

impl eframe::App for WhispoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for keyboard events
        let mut clear_receiver = false;
        let mut logs_to_add = Vec::new();
        
        if let Some(rx) = &self.keyboard_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    KeyboardEvent::RecordingKeyPressed => {
                        if !self.recording {
                            self.recording = true;
                            let shortcut_str = format_shortcut(&self.config.recording_shortcut);
                            let msg = match self.config.recording_shortcut.mode {
                                ShortcutMode::Hold => format!("{} pressed - Recording started", shortcut_str),
                                ShortcutMode::Toggle => format!("{} pressed - Recording started", shortcut_str),
                            };
                            logs_to_add.push(msg);
                        }
                    }
                    KeyboardEvent::RecordingKeyReleased => {
                        if self.recording {
                            self.recording = false;
                            let shortcut_str = format_shortcut(&self.config.recording_shortcut);
                            let msg = match self.config.recording_shortcut.mode {
                                ShortcutMode::Hold => format!("{} released - Recording stopped", shortcut_str),
                                ShortcutMode::Toggle => format!("{} pressed - Recording stopped", shortcut_str),
                            };
                            logs_to_add.push(msg);
                        }
                    }
                    KeyboardEvent::OtherKeyPressed => {
                        if self.recording {
                            self.recording = false;
                            logs_to_add.push("Recording cancelled".to_string());
                        }
                    }
                    KeyboardEvent::ListenerError(msg) => {
                        self.error_message = Some(msg.clone());
                        logs_to_add.push(format!("Keyboard listener error: {}", msg));
                        clear_receiver = true;
                    }
                }
            }
        }
        
        // Add logs after we're done borrowing
        for log in logs_to_add {
            self.add_log(log);
        }
        
        if clear_receiver {
            self.keyboard_rx = None;
        }
        
        // Request repaint for continuous updates
        ctx.request_repaint();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Whispo - Minimal Dictation App");
            
            ui.separator();
            
            // Show error message if any
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, format!("⚠️ {}", error));
                
                // Add buttons for permissions
                if !self.permissions_granted {
                    ui.horizontal(|ui| {
                        if ui.button("Open System Settings").clicked() {
                            self.open_accessibility_settings();
                        }
                        
                        if ui.button("Retry Permissions Check").clicked() {
                            self.init_keyboard_listener();
                        }
                    });
                }
                ui.separator();
            }
            
            // Recording status
            ui.horizontal(|ui| {
                ui.label("Status:");
                if self.recording {
                    ui.colored_label(egui::Color32::RED, "● RECORDING");
                } else if self.permissions_granted {
                    ui.colored_label(egui::Color32::GREEN, "● Ready");
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "● Permissions Required");
                }
            });
            
            ui.separator();
            
            // Configuration section
            ui.collapsing("Configuration", |ui| {
                ui.group(|ui| {
                    ui.label("STT Provider:");
                    ui.horizontal(|ui| {
                        if ui.radio(matches!(self.config.stt_provider, SttProvider::OpenAI), "OpenAI").clicked() {
                            self.config.stt_provider = SttProvider::OpenAI;
                            self.add_log("Changed STT provider to OpenAI".to_string());
                            let _ = self.config.save();
                        }
                        if ui.radio(matches!(self.config.stt_provider, SttProvider::Groq), "Groq").clicked() {
                            self.config.stt_provider = SttProvider::Groq;
                            self.add_log("Changed STT provider to Groq".to_string());
                            let _ = self.config.save();
                        }
                        #[cfg(target_os = "macos")]
                        if ui.radio(matches!(self.config.stt_provider, SttProvider::LightningWhisper), "Lightning Whisper").clicked() {
                            self.config.stt_provider = SttProvider::LightningWhisper;
                            self.add_log("Changed STT provider to Lightning Whisper".to_string());
                            let _ = self.config.save();
                        }
                    });
                });
                
                ui.add_space(10.0);
                
                // API Keys
                ui.group(|ui| {
                    ui.label("API Keys:");
                    
                    ui.horizontal(|ui| {
                        ui.label("OpenAI:");
                        let mut key = self.config.openai_api_key.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut key).changed() {
                            self.config.openai_api_key = if key.is_empty() { None } else { Some(key) };
                            self.add_log("Updated OpenAI API key".to_string());
                            let _ = self.config.save();
                        }
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Groq:");
                        let mut key = self.config.groq_api_key.clone().unwrap_or_default();
                        if ui.text_edit_singleline(&mut key).changed() {
                            self.config.groq_api_key = if key.is_empty() { None } else { Some(key) };
                            self.add_log("Updated Groq API key".to_string());
                            let _ = self.config.save();
                        }
                    });
                });
                
                ui.add_space(10.0);
                
                // Recording shortcut
                ui.group(|ui| {
                    ui.label("Recording Shortcut:");
                    
                    // Common shortcuts
                    ui.label("Quick presets:");
                    ui.horizontal(|ui| {
                        if ui.button("Hold Ctrl").clicked() {
                            self.config.recording_shortcut = RecordingShortcut {
                                mode: ShortcutMode::Hold,
                                key: KeyCode::ControlLeft,
                                modifiers: vec![],
                            };
                            self.add_log("Changed shortcut to Hold Ctrl".to_string());
                            let _ = self.config.save();
                            self.keyboard_rx = None;
                            self.init_keyboard_listener();
                        }
                        if ui.button("Ctrl+/").clicked() {
                            self.config.recording_shortcut = RecordingShortcut {
                                mode: ShortcutMode::Toggle,
                                key: KeyCode::Slash,
                                modifiers: vec![KeyCode::ControlLeft],
                            };
                            self.add_log("Changed shortcut to Ctrl+/".to_string());
                            let _ = self.config.save();
                            self.keyboard_rx = None;
                            self.init_keyboard_listener();
                        }
                        if ui.button("Cmd+Space").clicked() {
                            self.config.recording_shortcut = RecordingShortcut {
                                mode: ShortcutMode::Toggle,
                                key: KeyCode::Space,
                                modifiers: vec![KeyCode::MetaLeft],
                            };
                            self.add_log("Changed shortcut to Cmd+Space".to_string());
                            let _ = self.config.save();
                            self.keyboard_rx = None;
                            self.init_keyboard_listener();
                        }
                    });
                    
                    ui.separator();
                    
                    // Current shortcut display
                    ui.horizontal(|ui| {
                        ui.label("Current:");
                        ui.label(format_shortcut(&self.config.recording_shortcut));
                    });
                    
                    // Mode selection
                    ui.horizontal(|ui| {
                        ui.label("Mode:");
                        if ui.radio_value(&mut self.config.recording_shortcut.mode, ShortcutMode::Hold, "Hold").clicked() {
                            self.add_log("Changed mode to Hold".to_string());
                            let _ = self.config.save();
                            self.keyboard_rx = None;
                            self.init_keyboard_listener();
                        }
                        if ui.radio_value(&mut self.config.recording_shortcut.mode, ShortcutMode::Toggle, "Toggle").clicked() {
                            self.add_log("Changed mode to Toggle".to_string());
                            let _ = self.config.save();
                            self.keyboard_rx = None;
                            self.init_keyboard_listener();
                        }
                    });
                });
            });
            
            ui.separator();
            
            // Logs section
            ui.collapsing("Logs", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for log in self.logs.iter().rev() {
                            ui.label(log);
                        }
                    });
            });
        });
    }
}

fn format_shortcut(shortcut: &RecordingShortcut) -> String {
    let mut parts = Vec::new();
    
    // Add modifiers first
    for modifier in &shortcut.modifiers {
        parts.push(keycode_to_string(modifier));
    }
    
    // Add main key
    parts.push(keycode_to_string(&shortcut.key));
    
    // Join with + for key combinations
    parts.join("+")
}

fn keycode_to_string(keycode: &KeyCode) -> String {
    match keycode {
        KeyCode::ControlLeft | KeyCode::ControlRight => "Ctrl",
        KeyCode::ShiftLeft | KeyCode::ShiftRight => "Shift",
        KeyCode::Alt => "Alt",
        KeyCode::AltGr => "AltGr",
        KeyCode::MetaLeft | KeyCode::MetaRight => {
            #[cfg(target_os = "macos")]
            { "Cmd" }
            #[cfg(not(target_os = "macos"))]
            { "Meta" }
        }
        KeyCode::Space => "Space",
        KeyCode::Tab => "Tab",
        KeyCode::Return => "Enter",
        KeyCode::Escape => "Esc",
        KeyCode::Backspace => "Backspace",
        KeyCode::Delete => "Delete",
        KeyCode::Insert => "Insert",
        KeyCode::Home => "Home",
        KeyCode::End => "End",
        KeyCode::PageUp => "PageUp",
        KeyCode::PageDown => "PageDown",
        KeyCode::CapsLock => "CapsLock",
        KeyCode::UpArrow => "↑",
        KeyCode::DownArrow => "↓",
        KeyCode::LeftArrow => "←",
        KeyCode::RightArrow => "→",
        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",
        KeyCode::A => "A",
        KeyCode::B => "B",
        KeyCode::C => "C",
        KeyCode::D => "D",
        KeyCode::E => "E",
        KeyCode::F => "F",
        KeyCode::G => "G",
        KeyCode::H => "H",
        KeyCode::I => "I",
        KeyCode::J => "J",
        KeyCode::K => "K",
        KeyCode::L => "L",
        KeyCode::M => "M",
        KeyCode::N => "N",
        KeyCode::O => "O",
        KeyCode::P => "P",
        KeyCode::Q => "Q",
        KeyCode::R => "R",
        KeyCode::S => "S",
        KeyCode::T => "T",
        KeyCode::U => "U",
        KeyCode::V => "V",
        KeyCode::W => "W",
        KeyCode::X => "X",
        KeyCode::Y => "Y",
        KeyCode::Z => "Z",
        KeyCode::Num0 => "0",
        KeyCode::Num1 => "1",
        KeyCode::Num2 => "2",
        KeyCode::Num3 => "3",
        KeyCode::Num4 => "4",
        KeyCode::Num5 => "5",
        KeyCode::Num6 => "6",
        KeyCode::Num7 => "7",
        KeyCode::Num8 => "8",
        KeyCode::Num9 => "9",
        KeyCode::Slash => "/",
        KeyCode::BackSlash => "\\",
        KeyCode::Equal => "=",
        KeyCode::Minus => "-",
        KeyCode::Comma => ",",
        KeyCode::Dot => ".",
        KeyCode::SemiColon => ";",
        KeyCode::Quote => "'",
        KeyCode::LeftBracket => "[",
        KeyCode::RightBracket => "]",
        KeyCode::BackQuote => "`",
    }.to_string()
}