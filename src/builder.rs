// src/builder.rs
use std::ops::{Deref, DerefMut};
use dioxus::prelude::*;
use crate::state;
// use crate::markdown;

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
    /// Creates a new EditorBuilder with the given UCI coroutine and state.
    pub fn new(uci_action_tx: Option<Coroutine<String>>, state: State) -> Self {
        println!("{:?}", "New Editor");
        Self {
            uci_action_tx,
            state,
        }
    }

    /// Returns the raw text as a single concatenated string, joining all div texts with spaces.
    // pub fn get_raw_text(&self) -> String {
    //     self.state.raw_text.join(" ")
    // }

    /// Returns a reference to the raw text vector.
    pub fn get_raw_text_vec(&self) -> &Vec<Vec<String>> {
        &self.state.raw_text
    }

    // pub fn get_text_at(&self, index_i) -> String {
    //     if let Some((index_i, index_j, _)) = self.get_caret_pos() {
    //         self.state.raw_text[index_i][index_j].to_string()
    //     }
    //     else {
    //         String::from("error")
    //     }
    // }

    pub fn get_raw_text_current(&self) -> String {
        if let Some((index_i, index_j, _)) = self.get_caret_pos() {
            self.state.raw_text[index_i][index_j].to_string()
        }
        else {
            String::from("error")
        }
    }

    /// Updates the text at the specified index and adjusts the caret position if necessary.
    pub fn update_text(&mut self, index_i: usize, index_j: usize, text: String) {
        self.state.update_text(index_i, index_j, text);
    }

    /// Moves the caret to the specified position in the given text index.
    pub fn move_caret(&mut self, index_i: usize, index_j: usize, char_pos: usize) {
        self.state.move_caret(index_i, index_j, char_pos);
    }

    /// Clears the caret position, setting it to None.
    pub fn clear_caret(&mut self) {
        self.state.clear_caret();
    }

    /// Returns the current caret position.
    pub fn get_caret_pos(&self) -> Option<(usize, usize, usize)> {
        self.state.caret_pos
    }

    /// Sends a command to the UCI engine via the coroutine, if available.
    pub fn send_uci_command(&self, command: &str) {
        if let Some(tx) = &self.uci_action_tx {
            tx.send(command.to_string());
        }
    }

    pub fn get_row_level_caret_pos(&self, current_caret_pos: Option<(usize, usize, usize)>) -> Option<usize> {
        if let Some((index_i, index_j, char_pos)) = current_caret_pos {
            let row = &self.state.raw_text[index_i];
            // Sum lengths of all texts in columns before index_j
            let mut total_pos = 0;
            for j in 0..index_j {
                total_pos += row[j].len();
            }
            // Add the current cursor position
            total_pos += char_pos;
            Some(total_pos)
        } else {
            None
        }
    }

    pub fn get_caret_from_row_level_pos(&self, row_level_pos: usize, index_i: usize, row: Vec<String>) -> Option<(usize, usize, usize)> {
        // if index_i >= self.state.raw_text.len() {
        //     return None; // Invalid row index
        // }
    
        // let row = &self.state.raw_text[index_i];
        let mut remaining_pos = row_level_pos;
    
        for index_j in 0..row.len() {
            let text_len = row[index_j].len();
            if remaining_pos < text_len {
                // The position falls within this column
                return Some((index_i, index_j, remaining_pos + 1));
            }
            remaining_pos -= text_len;
        }
    
        // If the position exceeds the total length of the row, return the last column's end
        if !row.is_empty() {
            let last_j = row.len() - 1;
            let last_text_len = row[last_j].len();
            Some((index_i, last_j, last_text_len))
        } else {
            None // Empty row
        }
    }

}