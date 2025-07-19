use crate::handler;

// src/state.rs
use dioxus::prelude::*;

use crate::syntax::text::{TextProcessor, MarkDownElements};
use crate::syntax::markdown::{MarkDownStyle};


/// Holds all “global” editor signals and the caret coroutine.
/// Each `Signal<T>` is a reactive state that can be read or written.
#[derive(Clone)]
pub struct State {
    /// The main text buffer (e.g., Markdown, plain text, etc.)
    pub raw_text: Vec<Vec<String>>,

    /// Where the caret (cursor) is, if anywhere. `None` means no focus.
    pub caret_pos: Option<(usize, usize, usize)>,

    pub text_width: Vec<Vec<Option<f64>>>,


}

impl State {
    /// Constructs a new `State`, calling Dioxus hooks (use_signal, use_coroutine).
    /// **Must** be called inside a Dioxus component, passing the `cx: &ScopeState`.
    pub fn new(
        input_text: String,
        caret_pos: Option<(usize, usize, usize)>
    ) -> Self {
        
        let text_processor = TextProcessor::new();
        let syntax_text = text_processor.process_markdown(input_text);
        let raw_text = text_processor.extract_strings(syntax_text);
        let mut raw_text = raw_text.clone();
        while raw_text.len() < 20 {
            raw_text.push(vec![String::new()]);
        }

        let text_width: Vec<Vec<Option<f64>>> = raw_text.iter()
            .map(|inner| vec![None; inner.len()])
            .collect();

        // let selection_range = None::<Vec<(usize, usize)>>;
        println!("{:?}", "New State");
        Self {
            raw_text,
            caret_pos,
            text_width,
            // selection_range,
        }
    }

    /// Updates the text at the given indices and adjusts caret position if needed.
    pub fn update_text(&mut self, index_i: usize, index_j: usize, text: String) {
        while index_i >= self.raw_text.len() {
            self.raw_text.push(Vec::new());
        }
        while index_j >= self.raw_text[index_i].len() {
            self.raw_text[index_i].push("".to_string());
        }
        self.raw_text[index_i][index_j] = text;
    
        // Adjust caret position if it exists and is in the modified div
        if let Some((caret_i, caret_j, caret_offset)) = self.caret_pos {
            if caret_i == index_i && caret_j == index_j {
                self.caret_pos = Some((caret_i, caret_j, caret_offset.min(self.raw_text[index_i][index_j].len())));
            }
        }
    }

    /// Moves the caret to a new position, ensuring it's within valid bounds.
    pub fn move_caret(&mut self, index_i: usize, index_j: usize, char_pos: usize) {
        if index_i < self.raw_text.len() && index_j < self.raw_text[index_i].len() {
            self.caret_pos = Some((index_i, index_j, char_pos));
        } else {
            self.caret_pos = None;
        }
    }

    /// Clears the caret position.
    pub fn clear_caret(&mut self) {
        self.caret_pos = None;
    }
    
}
