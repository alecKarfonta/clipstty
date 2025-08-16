//! Interactive Audio Recording Menu System
//! 
//! This module provides a comprehensive interactive menu system for navigating
//! audio recording features, with voice commands and keyboard navigation.

use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Duration;
use chrono::{DateTime, Utc};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use uuid::Uuid;

use super::audio_archive::{
    AudioArchiveService, AudioError, RecordingSession, SearchCriteria, 
    StorageStats, SessionId, RecordingStatus
};

/// Interactive audio menu system
pub struct AudioRecordingMenu {
    /// Audio archive service
    audio_service: AudioArchiveService,
    /// Current menu state
    current_menu: MenuState,
    /// Menu history for navigation
    menu_history: Vec<MenuState>,
    /// Selected item index
    selected_index: usize,
    /// Menu configuration
    config: MenuConfig,
    /// Display buffer for smooth updates
    display_buffer: Vec<String>,
}

/// Menu state enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    MainMenu,
    RecordingMenu,
    SessionManagement,
    PlaybackMenu,
    StorageManagement,
    Settings,
    Help,
    RecordingInProgress(SessionId),
    SessionDetails(SessionId),
    SearchSessions,
    CompressionMenu,
    CleanupMenu,
}

/// Menu configuration
#[derive(Debug, Clone)]
pub struct MenuConfig {
    /// Enable voice commands
    pub voice_commands_enabled: bool,
    /// Auto-refresh interval
    pub auto_refresh_interval: Duration,
    /// Display colors
    pub use_colors: bool,
    /// Show help hints
    pub show_help_hints: bool,
    /// Animation speed
    pub animation_speed: Duration,
}

/// Menu item structure
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub shortcut: Option<char>,
    pub voice_command: Option<String>,
    pub action: MenuAction,
    pub enabled: bool,
}

/// Menu action enumeration
#[derive(Debug, Clone)]
pub enum MenuAction {
    NavigateTo(MenuState),
    StartRecording,
    StopRecording,
    PauseRecording,
    ResumeRecording,
    ListSessions,
    DeleteSession(SessionId),
    PlaySession(SessionId),
    CompressFiles,
    CleanupStorage,
    ShowStats,
    SearchSessions,
    ShowHelp,
    Exit,
    Custom(String),
}

/// Display theme for the menu
#[derive(Debug, Clone)]
pub struct DisplayTheme {
    pub primary_color: Color,
    pub secondary_color: Color,
    pub accent_color: Color,
    pub warning_color: Color,
    pub success_color: Color,
    pub error_color: Color,
}

impl AudioRecordingMenu {
    /// Create a new audio recording menu
    pub fn new(audio_service: AudioArchiveService) -> Self {
        Self {
            audio_service,
            current_menu: MenuState::MainMenu,
            menu_history: Vec::new(),
            selected_index: 0,
            config: MenuConfig::default(),
            display_buffer: Vec::new(),
        }
    }
    
    /// Start the interactive menu system
    pub fn run(&mut self) -> Result<(), AudioError> {
        // Initialize terminal
        terminal::enable_raw_mode().map_err(|e| AudioError::RecordingError(e.to_string()))?;
        
        let mut stdout = io::stdout();
        execute!(stdout, terminal::Clear(ClearType::All), cursor::Hide)
            .map_err(|e| AudioError::RecordingError(e.to_string()))?;
        
        // Main menu loop
        loop {
            self.render_current_menu()?;
            
            if let Ok(event) = event::read() {
                match self.handle_input(event)? {
                    MenuResult::Continue => continue,
                    MenuResult::Exit => break,
                    MenuResult::Error(msg) => {
                        self.show_error(&msg)?;
                    }
                }
            }
        }
        
        // Cleanup terminal
        execute!(stdout, cursor::Show, ResetColor)
            .map_err(|e| AudioError::RecordingError(e.to_string()))?;
        terminal::disable_raw_mode().map_err(|e| AudioError::RecordingError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Render the current menu
    fn render_current_menu(&mut self) -> Result<(), AudioError> {
        let mut stdout = io::stdout();
        execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))
            .map_err(|e| AudioError::RecordingError(e.to_string()))?;
        
        // Render header
        self.render_header(&mut stdout)?;
        
        // Render menu content based on current state
        match &self.current_menu {
            MenuState::MainMenu => self.render_main_menu(&mut stdout)?,
            MenuState::RecordingMenu => self.render_recording_menu(&mut stdout)?,
            MenuState::SessionManagement => self.render_session_management(&mut stdout)?,
            MenuState::PlaybackMenu => self.render_playback_menu(&mut stdout)?,
            MenuState::StorageManagement => self.render_storage_management(&mut stdout)?,
            MenuState::Settings => self.render_settings_menu(&mut stdout)?,
            MenuState::Help => self.render_help_menu(&mut stdout)?,
            MenuState::RecordingInProgress(session_id) => self.render_recording_progress(&mut stdout, *session_id)?,
            MenuState::SessionDetails(session_id) => self.render_session_details(&mut stdout, *session_id)?,
            MenuState::SearchSessions => self.render_search_sessions(&mut stdout)?,
            MenuState::CompressionMenu => self.render_compression_menu(&mut stdout)?,
            MenuState::CleanupMenu => self.render_cleanup_menu(&mut stdout)?,
        }
        
        // Render footer
        self.render_footer(&mut stdout)?;
        
        stdout.flush().map_err(|e| AudioError::RecordingError(e.to_string()))?;
        Ok(())
    }
    
    /// Render the header
    fn render_header(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        let theme = DisplayTheme::default();
        
        execute!(stdout, SetForegroundColor(theme.primary_color))?;
        execute!(stdout, Print("╭─────────────────────────────────────────────────────────────────────────╮\n"))?;
        execute!(stdout, Print("│                    🎙️  STT Clippy Audio Recording System                │\n"))?;
        execute!(stdout, Print("╰─────────────────────────────────────────────────────────────────────────╯\n"))?;
        execute!(stdout, ResetColor)?;
        
        // Show current recording status
        let status = self.audio_service.get_recording_status();
        if status.is_recording {
            execute!(stdout, SetForegroundColor(theme.error_color))?;
            execute!(stdout, Print("🔴 RECORDING IN PROGRESS"))?;
            if let Some(session) = &status.current_session {
                execute!(stdout, Print(&format!(" - {} ({:.1}s)", session.name, status.duration.as_secs_f64())))?;
            }
            execute!(stdout, Print("\n"))?;
            execute!(stdout, ResetColor)?;
        }
        
        execute!(stdout, Print("\n"))?;
        Ok(())
    }
    
    /// Render the main menu
    fn render_main_menu(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        let theme = DisplayTheme::default();
        
        execute!(stdout, SetForegroundColor(theme.accent_color))?;
        execute!(stdout, Print("📋 MAIN MENU\n\n"))?;
        execute!(stdout, ResetColor)?;
        
        let menu_items = vec![
            MenuItem {
                id: "recording".to_string(),
                title: "🎙️  Start Recording".to_string(),
                description: "Begin a new audio recording session".to_string(),
                shortcut: Some('r'),
                voice_command: Some("start recording".to_string()),
                action: MenuAction::NavigateTo(MenuState::RecordingMenu),
                enabled: !self.audio_service.get_recording_status().is_recording,
            },
            MenuItem {
                id: "sessions".to_string(),
                title: "📁 Session Management".to_string(),
                description: "View, manage, and organize recording sessions".to_string(),
                shortcut: Some('s'),
                voice_command: Some("manage sessions".to_string()),
                action: MenuAction::NavigateTo(MenuState::SessionManagement),
                enabled: true,
            },
            MenuItem {
                id: "playback".to_string(),
                title: "▶️  Playback & Review".to_string(),
                description: "Play back and review recorded audio".to_string(),
                shortcut: Some('p'),
                voice_command: Some("playback menu".to_string()),
                action: MenuAction::NavigateTo(MenuState::PlaybackMenu),
                enabled: true,
            },
            MenuItem {
                id: "storage".to_string(),
                title: "💾 Storage Management".to_string(),
                description: "Manage storage, compression, and cleanup".to_string(),
                shortcut: Some('m'),
                voice_command: Some("storage management".to_string()),
                action: MenuAction::NavigateTo(MenuState::StorageManagement),
                enabled: true,
            },
            MenuItem {
                id: "settings".to_string(),
                title: "⚙️  Settings".to_string(),
                description: "Configure audio recording preferences".to_string(),
                shortcut: Some('c'),
                voice_command: Some("settings".to_string()),
                action: MenuAction::NavigateTo(MenuState::Settings),
                enabled: true,
            },
            MenuItem {
                id: "help".to_string(),
                title: "❓ Help & Commands".to_string(),
                description: "View help and available voice commands".to_string(),
                shortcut: Some('h'),
                voice_command: Some("help".to_string()),
                action: MenuAction::NavigateTo(MenuState::Help),
                enabled: true,
            },
            MenuItem {
                id: "exit".to_string(),
                title: "🚪 Exit".to_string(),
                description: "Exit the audio recording system".to_string(),
                shortcut: Some('q'),
                voice_command: Some("exit".to_string()),
                action: MenuAction::Exit,
                enabled: true,
            },
        ];
        
        self.render_menu_items(stdout, &menu_items)?;
        Ok(())
    }
    
    /// Render the recording menu
    fn render_recording_menu(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        let theme = DisplayTheme::default();
        
        execute!(stdout, SetForegroundColor(theme.accent_color))?;
        execute!(stdout, Print("🎙️  RECORDING MENU\n\n"))?;
        execute!(stdout, ResetColor)?;
        
        let status = self.audio_service.get_recording_status();
        
        if status.is_recording {
            // Show recording controls
            execute!(stdout, SetForegroundColor(theme.error_color))?;
            execute!(stdout, Print("🔴 Recording in progress...\n\n"))?;
            execute!(stdout, ResetColor)?;
            
            if let Some(session) = &status.current_session {
                execute!(stdout, Print(&format!("Session: {}\n", session.name)))?;
                execute!(stdout, Print(&format!("Duration: {:.1} seconds\n", status.duration.as_secs_f64())))?;
                execute!(stdout, Print(&format!("Format: {:?} @ {}Hz\n\n", session.format_info.format, session.format_info.sample_rate)))?;
            }
            
            let menu_items = vec![
                MenuItem {
                    id: "pause".to_string(),
                    title: "⏸️  Pause Recording".to_string(),
                    description: "Temporarily pause the current recording".to_string(),
                    shortcut: Some('p'),
                    voice_command: Some("pause recording".to_string()),
                    action: MenuAction::PauseRecording,
                    enabled: true,
                },
                MenuItem {
                    id: "stop".to_string(),
                    title: "⏹️  Stop Recording".to_string(),
                    description: "Stop and save the current recording".to_string(),
                    shortcut: Some('s'),
                    voice_command: Some("stop recording".to_string()),
                    action: MenuAction::StopRecording,
                    enabled: true,
                },
            ];
            
            self.render_menu_items(stdout, &menu_items)?;
        } else {
            // Show start recording options
            execute!(stdout, Print("Start a new recording session:\n\n"))?;
            
            let menu_items = vec![
                MenuItem {
                    id: "quick_start".to_string(),
                    title: "🚀 Quick Start Recording".to_string(),
                    description: "Start recording with default settings".to_string(),
                    shortcut: Some('q'),
                    voice_command: Some("quick start recording".to_string()),
                    action: MenuAction::StartRecording,
                    enabled: true,
                },
                MenuItem {
                    id: "custom_start".to_string(),
                    title: "⚙️  Custom Recording Setup".to_string(),
                    description: "Configure recording settings before starting".to_string(),
                    shortcut: Some('c'),
                    voice_command: Some("custom recording".to_string()),
                    action: MenuAction::Custom("custom_recording".to_string()),
                    enabled: true,
                },
            ];
            
            self.render_menu_items(stdout, &menu_items)?;
        }
        
        Ok(())
    }
    
    /// Render session management menu
    fn render_session_management(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        let theme = DisplayTheme::default();
        
        execute!(stdout, SetForegroundColor(theme.accent_color))?;
        execute!(stdout, Print("📁 SESSION MANAGEMENT\n\n"))?;
        execute!(stdout, ResetColor)?;
        
        // Get recent sessions
        let sessions = self.audio_service.list_sessions(None).unwrap_or_default();
        let recent_sessions: Vec<_> = sessions.iter().take(10).collect();
        
        execute!(stdout, Print(&format!("Total Sessions: {}\n\n", sessions.len())))?;
        
        if !recent_sessions.is_empty() {
            execute!(stdout, Print("Recent Sessions:\n"))?;
            execute!(stdout, Print("─".repeat(75)))?;
            execute!(stdout, Print("\n"))?;
            
            for (i, session) in recent_sessions.iter().enumerate() {
                let duration_str = format!("{:.1}s", session.duration.as_secs_f64());
                let size_str = format!("{:.1}MB", session.file_size as f64 / 1024.0 / 1024.0);
                let date_str = session.start_time.format("%Y-%m-%d %H:%M").to_string();
                
                if i == self.selected_index {
                    execute!(stdout, SetForegroundColor(theme.accent_color))?;
                    execute!(stdout, Print("► "))?;
                } else {
                    execute!(stdout, Print("  "))?;
                }
                
                execute!(stdout, Print(&format!("{:<30} {} {} {}\n", 
                    session.name, duration_str, size_str, date_str)))?;
                execute!(stdout, ResetColor)?;
            }
            
            execute!(stdout, Print("\n"))?;
        }
        
        let menu_items = vec![
            MenuItem {
                id: "search".to_string(),
                title: "🔍 Search Sessions".to_string(),
                description: "Search for specific recording sessions".to_string(),
                shortcut: Some('s'),
                voice_command: Some("search sessions".to_string()),
                action: MenuAction::NavigateTo(MenuState::SearchSessions),
                enabled: true,
            },
            MenuItem {
                id: "list_all".to_string(),
                title: "📋 List All Sessions".to_string(),
                description: "View all recording sessions".to_string(),
                shortcut: Some('l'),
                voice_command: Some("list all sessions".to_string()),
                action: MenuAction::ListSessions,
                enabled: true,
            },
        ];
        
        self.render_menu_items(stdout, &menu_items)?;
        Ok(())
    }
    
    /// Render storage management menu
    fn render_storage_management(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        let theme = DisplayTheme::default();
        
        execute!(stdout, SetForegroundColor(theme.accent_color))?;
        execute!(stdout, Print("💾 STORAGE MANAGEMENT\n\n"))?;
        execute!(stdout, ResetColor)?;
        
        // Show storage statistics
        let stats = self.audio_service.get_storage_stats();
        
        execute!(stdout, Print("Storage Statistics:\n"))?;
        execute!(stdout, Print("─".repeat(50)))?;
        execute!(stdout, Print("\n"))?;
        execute!(stdout, Print(&format!("Total Sessions: {}\n", stats.total_sessions)))?;
        execute!(stdout, Print(&format!("Total Size: {:.2} MB\n", stats.total_size_bytes as f64 / 1024.0 / 1024.0)))?;
        execute!(stdout, Print(&format!("Total Duration: {:.1} minutes\n", stats.total_duration.as_secs_f64() / 60.0)))?;
        execute!(stdout, Print(&format!("Compression Ratio: {:.1}%\n", stats.compression_ratio * 100.0)))?;
        
        if let Some(oldest) = stats.oldest_session {
            execute!(stdout, Print(&format!("Oldest Session: {}\n", oldest.format("%Y-%m-%d"))))?;
        }
        
        execute!(stdout, Print("\n"))?;
        
        let menu_items = vec![
            MenuItem {
                id: "compress".to_string(),
                title: "🗜️  Compress Audio Files".to_string(),
                description: "Compress all audio files to save space".to_string(),
                shortcut: Some('c'),
                voice_command: Some("compress files".to_string()),
                action: MenuAction::NavigateTo(MenuState::CompressionMenu),
                enabled: true,
            },
            MenuItem {
                id: "cleanup".to_string(),
                title: "🧹 Cleanup Old Files".to_string(),
                description: "Remove old files based on retention policy".to_string(),
                shortcut: Some('u'),
                voice_command: Some("cleanup storage".to_string()),
                action: MenuAction::NavigateTo(MenuState::CleanupMenu),
                enabled: true,
            },
            MenuItem {
                id: "stats".to_string(),
                title: "📊 Detailed Statistics".to_string(),
                description: "View detailed storage and usage statistics".to_string(),
                shortcut: Some('d'),
                voice_command: Some("show stats".to_string()),
                action: MenuAction::ShowStats,
                enabled: true,
            },
        ];
        
        self.render_menu_items(stdout, &menu_items)?;
        Ok(())
    }
    
    /// Render help menu
    fn render_help_menu(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        let theme = DisplayTheme::default();
        
        execute!(stdout, SetForegroundColor(theme.accent_color))?;
        execute!(stdout, Print("❓ HELP & VOICE COMMANDS\n\n"))?;
        execute!(stdout, ResetColor)?;
        
        execute!(stdout, Print("Keyboard Navigation:\n"))?;
        execute!(stdout, Print("─".repeat(50)))?;
        execute!(stdout, Print("\n"))?;
        execute!(stdout, Print("↑/↓ or j/k    Navigate menu items\n"))?;
        execute!(stdout, Print("Enter/Space   Select item\n"))?;
        execute!(stdout, Print("Esc/Backspace Go back\n"))?;
        execute!(stdout, Print("q             Quit\n"))?;
        execute!(stdout, Print("Letter keys   Quick shortcuts\n\n"))?;
        
        execute!(stdout, Print("Voice Commands:\n"))?;
        execute!(stdout, Print("─".repeat(50)))?;
        execute!(stdout, Print("\n"))?;
        execute!(stdout, Print("\"start recording\"     - Begin new recording session\n"))?;
        execute!(stdout, Print("\"stop recording\"      - End current recording\n"))?;
        execute!(stdout, Print("\"pause recording\"     - Pause current recording\n"))?;
        execute!(stdout, Print("\"resume recording\"    - Resume paused recording\n"))?;
        execute!(stdout, Print("\"list sessions\"       - Show all sessions\n"))?;
        execute!(stdout, Print("\"compress files\"      - Compress audio files\n"))?;
        execute!(stdout, Print("\"cleanup storage\"     - Clean up old files\n"))?;
        execute!(stdout, Print("\"show stats\"          - Display storage statistics\n"))?;
        execute!(stdout, Print("\"help\"                - Show this help\n"))?;
        execute!(stdout, Print("\"exit\"                - Exit the application\n\n"))?;
        
        Ok(())
    }
    
    /// Render menu items
    fn render_menu_items(&self, stdout: &mut io::Stdout, items: &[MenuItem]) -> Result<(), AudioError> {
        let theme = DisplayTheme::default();
        
        for (i, item) in items.iter().enumerate() {
            if i == self.selected_index {
                execute!(stdout, SetForegroundColor(theme.accent_color))?;
                execute!(stdout, Print("► "))?;
            } else {
                execute!(stdout, Print("  "))?;
            }
            
            if !item.enabled {
                execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
            }
            
            execute!(stdout, Print(&item.title))?;
            
            if let Some(shortcut) = item.shortcut {
                execute!(stdout, SetForegroundColor(theme.secondary_color))?;
                execute!(stdout, Print(&format!(" ({})", shortcut)))?;
            }
            
            execute!(stdout, ResetColor)?;
            execute!(stdout, Print("\n"))?;
            
            if self.config.show_help_hints {
                execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
                execute!(stdout, Print(&format!("    {}\n", item.description)))?;
                execute!(stdout, ResetColor)?;
            }
        }
        
        Ok(())
    }
    
    /// Render footer
    fn render_footer(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        let theme = DisplayTheme::default();
        
        execute!(stdout, Print("\n"))?;
        execute!(stdout, SetForegroundColor(theme.secondary_color))?;
        execute!(stdout, Print("─".repeat(75)))?;
        execute!(stdout, Print("\n"))?;
        
        match &self.current_menu {
            MenuState::MainMenu => {
                execute!(stdout, Print("Use ↑/↓ to navigate, Enter to select, 'q' to quit"))?;
            }
            _ => {
                execute!(stdout, Print("Use ↑/↓ to navigate, Enter to select, Esc to go back"))?;
            }
        }
        
        if self.config.voice_commands_enabled {
            execute!(stdout, Print(" | Voice commands enabled"))?;
        }
        
        execute!(stdout, ResetColor)?;
        Ok(())
    }
    
    /// Handle user input
    fn handle_input(&mut self, event: Event) -> Result<MenuResult, AudioError> {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event),
            _ => Ok(MenuResult::Continue),
        }
    }
    
    /// Handle keyboard input
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<MenuResult, AudioError> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                Ok(MenuResult::Continue)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                // This would need to be adjusted based on current menu items
                self.selected_index += 1;
                Ok(MenuResult::Continue)
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.execute_selected_action()
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.go_back()
            }
            KeyCode::Char('q') => {
                if self.current_menu == MenuState::MainMenu {
                    Ok(MenuResult::Exit)
                } else {
                    self.go_back()
                }
            }
            KeyCode::Char(c) => {
                self.handle_shortcut(c)
            }
            _ => Ok(MenuResult::Continue),
        }
    }
    
    /// Execute the currently selected action
    fn execute_selected_action(&mut self) -> Result<MenuResult, AudioError> {
        // This would execute the action based on current menu and selected index
        // For now, just navigate to recording menu as an example
        match &self.current_menu {
            MenuState::MainMenu => {
                match self.selected_index {
                    0 => self.navigate_to(MenuState::RecordingMenu),
                    1 => self.navigate_to(MenuState::SessionManagement),
                    2 => self.navigate_to(MenuState::PlaybackMenu),
                    3 => self.navigate_to(MenuState::StorageManagement),
                    4 => self.navigate_to(MenuState::Settings),
                    5 => self.navigate_to(MenuState::Help),
                    6 => Ok(MenuResult::Exit),
                    _ => Ok(MenuResult::Continue),
                }
            }
            MenuState::RecordingMenu => {
                if self.audio_service.get_recording_status().is_recording {
                    match self.selected_index {
                        0 => {
                            self.audio_service.pause_recording()?;
                            Ok(MenuResult::Continue)
                        }
                        1 => {
                            self.audio_service.stop_recording_session()?;
                            self.navigate_to(MenuState::MainMenu)
                        }
                        _ => Ok(MenuResult::Continue),
                    }
                } else {
                    match self.selected_index {
                        0 => self.start_quick_recording(),
                        1 => self.navigate_to(MenuState::Settings),
                        _ => Ok(MenuResult::Continue),
                    }
                }
            }
            _ => Ok(MenuResult::Continue),
        }
    }
    
    /// Start a quick recording session
    fn start_quick_recording(&mut self) -> Result<MenuResult, AudioError> {
        let session_name = format!("Recording {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
        let session_id = self.audio_service.start_recording_session(session_name, None)?;
        self.navigate_to(MenuState::RecordingInProgress(session_id))
    }
    
    /// Navigate to a different menu
    fn navigate_to(&mut self, new_state: MenuState) -> Result<MenuResult, AudioError> {
        self.menu_history.push(self.current_menu.clone());
        self.current_menu = new_state;
        self.selected_index = 0;
        Ok(MenuResult::Continue)
    }
    
    /// Go back to previous menu
    fn go_back(&mut self) -> Result<MenuResult, AudioError> {
        if let Some(previous_state) = self.menu_history.pop() {
            self.current_menu = previous_state;
            self.selected_index = 0;
        }
        Ok(MenuResult::Continue)
    }
    
    /// Handle keyboard shortcuts
    fn handle_shortcut(&mut self, c: char) -> Result<MenuResult, AudioError> {
        match &self.current_menu {
            MenuState::MainMenu => {
                match c {
                    'r' => self.navigate_to(MenuState::RecordingMenu),
                    's' => self.navigate_to(MenuState::SessionManagement),
                    'p' => self.navigate_to(MenuState::PlaybackMenu),
                    'm' => self.navigate_to(MenuState::StorageManagement),
                    'c' => self.navigate_to(MenuState::Settings),
                    'h' => self.navigate_to(MenuState::Help),
                    _ => Ok(MenuResult::Continue),
                }
            }
            _ => Ok(MenuResult::Continue),
        }
    }
    
    /// Show error message
    fn show_error(&self, message: &str) -> Result<(), AudioError> {
        let mut stdout = io::stdout();
        let theme = DisplayTheme::default();
        
        execute!(stdout, SetForegroundColor(theme.error_color))?;
        execute!(stdout, Print(&format!("\n❌ Error: {}\n", message)))?;
        execute!(stdout, Print("Press any key to continue..."))?;
        execute!(stdout, ResetColor)?;
        stdout.flush().map_err(|e| AudioError::RecordingError(e.to_string()))?;
        
        // Wait for key press
        let _ = event::read();
        
        Ok(())
    }
    
    /// Render recording progress (placeholder)
    fn render_recording_progress(&self, stdout: &mut io::Stdout, _session_id: SessionId) -> Result<(), AudioError> {
        execute!(stdout, Print("Recording in progress...\n"))?;
        Ok(())
    }
    
    /// Render session details (placeholder)
    fn render_session_details(&self, stdout: &mut io::Stdout, _session_id: SessionId) -> Result<(), AudioError> {
        execute!(stdout, Print("Session details...\n"))?;
        Ok(())
    }
    
    /// Render search sessions (placeholder)
    fn render_search_sessions(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        execute!(stdout, Print("Search sessions...\n"))?;
        Ok(())
    }
    
    /// Render compression menu (placeholder)
    fn render_compression_menu(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        execute!(stdout, Print("Compression menu...\n"))?;
        Ok(())
    }
    
    /// Render cleanup menu (placeholder)
    fn render_cleanup_menu(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        execute!(stdout, Print("Cleanup menu...\n"))?;
        Ok(())
    }
    
    /// Render settings menu (placeholder)
    fn render_settings_menu(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        execute!(stdout, Print("Settings menu...\n"))?;
        Ok(())
    }
    
    /// Render playback menu (placeholder)
    fn render_playback_menu(&self, stdout: &mut io::Stdout) -> Result<(), AudioError> {
        execute!(stdout, Print("Playback menu...\n"))?;
        Ok(())
    }
}

/// Menu operation result
#[derive(Debug)]
pub enum MenuResult {
    Continue,
    Exit,
    Error(String),
}

// Default implementations
impl Default for MenuConfig {
    fn default() -> Self {
        Self {
            voice_commands_enabled: true,
            auto_refresh_interval: Duration::from_millis(500),
            use_colors: true,
            show_help_hints: true,
            animation_speed: Duration::from_millis(100),
        }
    }
}

impl Default for DisplayTheme {
    fn default() -> Self {
        Self {
            primary_color: Color::Cyan,
            secondary_color: Color::Blue,
            accent_color: Color::Yellow,
            warning_color: Color::Magenta,
            success_color: Color::Green,
            error_color: Color::Red,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::audio_archive::{AudioArchiveConfig, AudioRecorder, AudioStorage, AudioFormatInfo, AudioFormat};
    use std::collections::HashMap;
    
    // Mock implementations for testing
    struct MockAudioRecorder;
    impl AudioRecorder for MockAudioRecorder {
        fn start_recording(&mut self) -> Result<(), AudioError> { Ok(()) }
        fn stop_recording(&mut self) -> Result<Vec<f32>, AudioError> { Ok(vec![]) }
        fn pause_recording(&mut self) -> Result<(), AudioError> { Ok(()) }
        fn resume_recording(&mut self) -> Result<(), AudioError> { Ok(()) }
        fn is_recording(&self) -> bool { false }
        fn get_recording_duration(&self) -> Duration { Duration::from_secs(0) }
        fn get_format_info(&self) -> AudioFormatInfo {
            AudioFormatInfo {
                sample_rate: 44100,
                channels: 1,
                bit_depth: 16,
                format: AudioFormat::WAV,
            }
        }
    }
    
    struct MockAudioStorage;
    impl AudioStorage for MockAudioStorage {
        fn store_audio(&mut self, _session: &RecordingSession, _data: &[f32]) -> Result<super::AudioFileId, AudioError> { Ok(Uuid::new_v4()) }
        fn retrieve_audio(&self, _file_id: super::AudioFileId) -> Result<Vec<f32>, AudioError> { Ok(vec![]) }
        fn list_sessions(&self, _criteria: SearchCriteria) -> Result<Vec<RecordingSession>, AudioError> { Ok(vec![]) }
        fn delete_session(&mut self, _session_id: SessionId) -> Result<(), AudioError> { Ok(()) }
        fn get_storage_stats(&self) -> super::StorageStats {
            super::StorageStats {
                total_sessions: 0,
                total_size_bytes: 0,
                total_duration: Duration::from_secs(0),
                compression_ratio: 0.0,
                oldest_session: None,
                newest_session: None,
            }
        }
        fn compress_audio_files(&mut self) -> Result<super::CompressionResult, AudioError> {
            Ok(super::CompressionResult {
                files_compressed: 0,
                original_size: 0,
                compressed_size: 0,
                compression_ratio: 0.0,
                time_taken: Duration::from_secs(0),
            })
        }
        fn cleanup_old_files(&mut self, _retention_policy: &super::RetentionPolicy) -> Result<super::CleanupResult, AudioError> {
            Ok(super::CleanupResult {
                files_deleted: 0,
                space_freed: 0,
                sessions_removed: 0,
            })
        }
    }
    
    #[test]
    fn test_menu_creation() {
        let recorder = Box::new(MockAudioRecorder);
        let storage = Box::new(MockAudioStorage);
        let config = AudioArchiveConfig::default();
        let audio_service = super::AudioArchiveService::new(recorder, storage, config).unwrap();
        let menu = AudioRecordingMenu::new(audio_service);
        
        assert_eq!(menu.current_menu, MenuState::MainMenu);
        assert_eq!(menu.selected_index, 0);
    }
}
