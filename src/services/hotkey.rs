//! Hotkey service for managing global hotkey registration and handling.

use crate::{core::types::*, Result};
use std::collections::HashSet;

/// Hotkey service for managing global hotkeys
pub struct HotkeyService {
    registered: HashSet<Hotkey>,
}

impl HotkeyService {
    /// Create a new hotkey service
    pub fn new() -> Result<Self> {
        Ok(Self {
            registered: HashSet::new(),
        })
    }

    /// Register a global hotkey
    pub fn register_hotkey(&mut self, hotkey: &Hotkey) -> Result<()> {
        self.registered.insert(hotkey.clone());
        Ok(())
    }

    /// Unregister a global hotkey
    pub fn unregister_hotkey(&mut self, hotkey: &Hotkey) -> Result<()> {
        self.registered.remove(hotkey);
        Ok(())
    }

    /// Check if hotkey is registered
    pub fn is_registered(&self, hotkey: &Hotkey) -> bool {
        self.registered.contains(hotkey)
    }
}
