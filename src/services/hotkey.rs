//! Hotkey service for managing global hotkey registration and handling.

use crate::{core::types::*, Result};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Hotkey service for managing global hotkeys
pub struct HotkeyService {
    registered: HashSet<Hotkey>,
    press_handlers: HashMap<Hotkey, Vec<Arc<dyn Fn() + 'static>>>,
    release_handlers: HashMap<Hotkey, Vec<Arc<dyn Fn() + 'static>>>,
}

impl HotkeyService {
    /// Create a new hotkey service
    pub fn new() -> Result<Self> {
        Ok(Self {
            registered: HashSet::new(),
            press_handlers: HashMap::new(),
            release_handlers: HashMap::new(),
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

    /// Register a handler for key press (key down)
    pub fn on_press(&mut self, hotkey: &Hotkey, handler: Arc<dyn Fn() + 'static>) {
        self.press_handlers
            .entry(hotkey.clone())
            .or_default()
            .push(handler);
    }

    /// Register a handler for key release (key up)
    pub fn on_release(&mut self, hotkey: &Hotkey, handler: Arc<dyn Fn() + 'static>) {
        self.release_handlers
            .entry(hotkey.clone())
            .or_default()
            .push(handler);
    }

    /// Trigger press handlers for a hotkey (to be called by platform layer)
    pub fn handle_press(&self, hotkey: &Hotkey) {
        if let Some(list) = self.press_handlers.get(hotkey) {
            for h in list {
                h();
            }
        }
    }

    /// Trigger release handlers for a hotkey (to be called by platform layer)
    pub fn handle_release(&self, hotkey: &Hotkey) {
        if let Some(list) = self.release_handlers.get(hotkey) {
            for h in list {
                h();
            }
        }
    }
}
