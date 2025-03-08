// src/builder.rs
use std::ops::{Deref, DerefMut};
use dioxus::prelude::*;
use crate::state;
use crate::markdown;

use state::State;
/// A “builder” struct that owns the `State`. You can add extra fields, too.
pub struct EditorBuilder {
    /// For example, some external coroutine (e.g., UCI engine comms).
    pub uci_action_tx: Option<Coroutine<String>>,

    /// The underlying state for this editor.
    state: State,
}


impl Deref for EditorBuilder {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for EditorBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl EditorBuilder {
    /// Create a new EditorBuilder. Must be called from within a Dioxus component,
    /// so we can pass `cx: &ScopeState` to `State::new(cx)`.
    
    pub fn new(uci_action_tx: Option<Coroutine<String>>, state: State) -> Self {
        println!("{:?}", "New Editor");
        Self {
            uci_action_tx,
            state: state,
        }
    }

    pub fn get_raw_text(&self) -> String {
        self.deref().raw_text.read().clone()
    }

    pub fn get_selection_range(&self) -> Option<(usize, usize)> {
        self.deref().selection_range.read().clone()
    }


    /// Convenience function to set caret position (calls `state.set_caret()`).
    pub fn get_caret_position(&self) -> Option<usize> {
        self.deref().caret_pos.read().clone()
    }

    /// Convenience function to set caret position (calls `state.set_caret()`).
    pub fn set_caret_position(&mut self, new_pos: usize) {
        self.deref_mut().set_caret(new_pos)
    }


    pub fn handle_keydown_event(&mut self, evt: KeyboardEvent){

        self.deref_mut().handle_keydown(evt);
        println!("Test");
    }

    // Add other builder/utility methods here...
}

