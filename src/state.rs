use crate::markdown;
use crate::handler;

// src/state.rs
use dioxus::prelude::*;

/// Holds all “global” editor signals and the caret coroutine.
/// Each `Signal<T>` is a reactive state that can be read or written.
pub struct State {
    /// The main text buffer (e.g., Markdown, plain text, etc.)
    pub raw_text: Signal<String>,

    /// Where the caret (cursor) is, if anywhere. `None` means no focus.
    pub caret_pos: Signal<Option<usize>>,

    /// An undo stack of previous `raw_text`s, for Ctrl+Z usage.
    pub undo_stack: Signal<Vec<String>>,

    /// A selection range (start, end) for text highlighting, if any.
    pub selection_range: Signal<Option<(usize, usize)>>,

    /// Tracks whether the editor was recently clicked.
    pub last_key_entered: Signal<Option<Key>>,
}

impl State {
    /// Constructs a new `State`, calling Dioxus hooks (use_signal, use_coroutine).
    /// **Must** be called inside a Dioxus component, passing the `cx: &ScopeState`.
    pub fn new(
        raw_text: Signal<String>,
        caret_pos: Signal<Option<usize>>,
        undo_stack: Signal<Vec<String>>,
        selection_range: Signal<Option<(usize, usize)>>,
        last_key_entered: Signal<Option<Key>>
    ) -> Self {
        println!("{:?}", "New State");
        Self {
            raw_text,
            caret_pos,
            undo_stack,
            selection_range,
            last_key_entered
        }
    }

    /// Convenience method to set the caret position (both the signal and the DOM).
    pub fn set_caret(&mut self, pos: usize) {

        self.caret_pos.set(Some(pos));
        // self.caret_queue.send(pos);
    }


    /// Example helper to update selection in one call.
    pub fn update_selection(&mut self, anchor: usize, active: usize) {
        self.selection_range.set(Some((anchor, active)));
    }




    pub fn handle_keydown(&mut self, evt: KeyboardEvent) {

        let read_text = self.raw_text.read().clone();
        
        // let read_text = read_text.replace("\r\n", ";nbp*p!");

        let read_selection_range = self.selection_range.read().clone();
        let pos = match *self.caret_pos.read() {
            Some(pos) => pos,
            None => return,
        };

       println!("Handling keydown at position: {}", pos);

    
        // Handle Ctrl shortcuts (Undo, Copy, Paste, Cut)
        if evt.data().modifiers().ctrl() {
            let key_lower = evt.data().key().to_string();
            match key_lower.as_str() {
                "z" => {
                    evt.prevent_default();
                    if let Some(previous) = self.undo_stack.write().pop() {
                        self.raw_text.set(previous);
                    }
                }
                "c" => {
                    evt.prevent_default();
                    if let Some((sel_start, sel_end)) = *self.selection_range.read() {
                        let s = sel_start.min(sel_end);
                        let e = sel_start.max(sel_end);
                        let selected_text = &self.raw_text.read()[s..e];
    
                        let mut clipboard = arboard::Clipboard::new().expect("Failed to open clipboard");
                        clipboard.set_text(selected_text.to_string()).expect("Failed to copy to clipboard");
                    }
                }
                "v" => {
                    evt.prevent_default();
                    let mut clipboard = arboard::Clipboard::new().expect("Failed to open clipboard");
                    if let Ok(paste_text) = clipboard.get_text() {
                        let pos = self.caret_pos.read().unwrap_or(0);
                        self.undo_stack.write().push(read_text.clone());
                        let (left, right) = read_text.split_at(pos);
                        let new_text = format!("{}{}{}", left, paste_text, right);
                        self.raw_text.set(new_text);
                        self.set_caret(pos + paste_text.len());
                    }
                }
                "x" => {
                    evt.prevent_default();
                    if let Some((sel_start, sel_end)) = read_selection_range {
                        let s = sel_start.min(sel_end);
                        let e = sel_start.max(sel_end);
                        let selected_text = &read_text[s..e];
    
                        let mut clipboard = arboard::Clipboard::new().expect("Failed to open clipboard");
                        clipboard.set_text(selected_text.to_string()).expect("Failed to copy to clipboard");
    
                        // Delete selected text after copying
                        self.undo_stack.write().push(read_text.clone());
                        let new_text = format!("{}{}", &read_text[0..s], &read_text[e..]);
                        self.raw_text.set(new_text);
                        self.set_caret(s);
                        self.selection_range.set(None);
                    }
                }
                _ => {}
            }
            return;
        }
    
        // Handle Shift-based text selection
        if evt.data().modifiers().shift() && matches!(evt.data().key(), Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End) {
            match evt.data().key() {
                Key::ArrowLeft => {
                    evt.prevent_default();
                    if pos > 0 {
                        let new_pos = pos - 1;
                        self.set_caret(new_pos);
                        self.selection_range.with_mut(|sel_range| {
                            if let Some((anchor, _)) = *sel_range {
                                *sel_range = Some((anchor, new_pos));
                            } else {
                                *sel_range = Some((pos, new_pos));
                            }
                        });
                    }
                }
                Key::ArrowRight => {
                    evt.prevent_default();
                    if pos < read_text.len() {
                        let new_pos = pos + 1;
                        self.set_caret(new_pos);
                        self.selection_range.with_mut(|sel_range| {
                            if let Some((anchor, _)) = *sel_range {
                                *sel_range = Some((anchor, new_pos));
                            } else {
                                *sel_range = Some((pos, new_pos));
                            }
                        });
                    }
                }
                _ => {}
            }
            return;
        }
    
        // Normal key handling
        match evt.data().key() {
            
            
            Key::Enter => {
                evt.prevent_default();
                
                self.undo_stack.write().push(read_text.clone());
                
                let last_key = (*self.last_key_entered.read()).clone();

                let mut last_key_was_enter = false;
                if last_key == Some(Key::Enter) {
                    last_key_was_enter = true;
                }
                println!("{:?}", last_key_was_enter);
                let (new_text, new_pos) = handler::handle_enter_key(&read_text, pos.clone(), last_key_was_enter);

                self.raw_text.set(new_text);
            
                self.set_caret(new_pos);
            }
            Key::Backspace => {
                evt.prevent_default();

                if pos > 0 {
                    self.undo_stack.write().push(read_text.clone());
                }

                if let Some((sel_start, sel_end)) = read_selection_range {
                    // Handle text selection deletion
                    let s = sel_start.min(sel_end);
                    let e = sel_start.max(sel_end);
                    let new_text = format!("{}{}", &read_text[0..s], &read_text[e..]);
                    self.raw_text.set(new_text);
                    self.set_caret(s);
                    self.selection_range.set(None);
                } else if pos > 0 {
                    let (new_text, new_pos) = handler::handle_backspace_no_selection(&read_text, pos);
                    self.raw_text.set(new_text);
                    self.set_caret(new_pos);
                }
            }
            Key::Delete => {
                evt.prevent_default();
                if let Some((sel_start, sel_end)) = read_selection_range {
                    let s = sel_start.min(sel_end);
                    let e = sel_start.max(sel_end);
                    self.undo_stack.write().push(read_text.clone());
                    let new_text = format!("{}{}", &read_text[0..s], &read_text[e..]);
                    self.raw_text.set(new_text);
                    self.set_caret(s);
                    self.selection_range.set(None);
                } else if pos < read_text.len() {
                    self.undo_stack.write().push(read_text.clone());
                    let mut chars: Vec<char> = read_text.chars().collect();
                    chars.remove(pos);
                    self.raw_text.set(chars.into_iter().collect());
                    self.set_caret(pos);
                }
            }
            Key::ArrowLeft => {
                evt.prevent_default();
                if pos > 0 {
                    self.set_caret(pos - 1);
                }
            }
            Key::ArrowRight => {
                evt.prevent_default();
                if pos < read_text.len() {
                    self.set_caret(pos + 1);
                }
            }
            _ => {
                evt.prevent_default();
                if let Key::Character(ch) = &evt.data.key() {
                    
                    self.undo_stack.write().push(read_text.clone());
                    
                    self.set_caret(pos);

                    let (new_text, new_pos) = handler::handle_character_input(&read_text, pos.clone(), ch);

                    self.raw_text.set(new_text);
                    self.set_caret(new_pos);
                }
            }
        }

        
        self.last_key_entered.set(Some(evt.data().key()));
        println!("{:?}", Some(evt.data().key()));

        println!("raw {:?}", self.raw_text.read().clone());
        // Reset selection if normal typing occurs
        self.selection_range.set(None);
    }
    
}
