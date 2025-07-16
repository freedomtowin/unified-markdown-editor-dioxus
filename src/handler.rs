use dioxus::prelude::*;
use crate::EditorBuilder;
use std::collections::VecDeque;

// Handler for Enter key
pub fn handle_enter_key(
    event: KeyboardEvent,
    index_i: usize,
    index_j: usize,
    editor: &Signal<EditorBuilder>,
    measure_width: Coroutine<(usize, usize, String)>,
    editor_updates: Signal<VecDeque<(usize, usize, usize, String)>>,
) -> Result<(), &'static str>{
    event.stop_propagation();
    event.prevent_default();
    
    let mut editor = editor.clone();
    let mut editor_updates = editor_updates.clone();
    spawn(async move {
        editor.with_mut(|e| {
            let (caret_i, caret_j, caret_pos) = e.get_caret_pos().unwrap_or((index_i, index_j, 0));
            if caret_i == index_i && caret_j == index_j {
                let current_text = e.raw_text[index_i][index_j].clone();
                let (before, after) = current_text.split_at(caret_pos);

                e.update_text(index_i, index_j, before.to_string());
                e.raw_text.insert(index_i + 1, vec![after.to_string()]);
                
                measure_width.send((index_i, index_j, before.to_string()));
                measure_width.send((index_i, index_j + 1, after.to_string()));

                editor_updates.write().push_back((index_i, index_j, before.len(), before.to_string()));
                editor_updates.write().push_back((index_i + 1, 0, 0, after.to_string()));
            }
        });
    });

    Ok(())
}


pub fn handle_character_input(current_text: &str, current_caret_pos: usize, key: Key) -> (String, usize) {
    let mut new_text = current_text.to_string();
    let mut new_caret_pos = current_caret_pos;

    if let Key::Character(ch) = &key
    {
        // Insert the character at the caret position
        let char_str = key.to_string();
        new_text.insert_str(current_caret_pos, &char_str);
        new_caret_pos += char_str.len();
    } else if key == Key::Backspace && current_caret_pos > 0 {
        // Remove the character before the caret
        new_text.remove(current_caret_pos - 1);
        new_caret_pos -= 1;
    }

    (new_text, new_caret_pos)
}


// pub fn handle_enter_key(read_text: &str, pos: usize, last_key_was_enter: bool) -> (String, usize) {
//     let (left, right) = read_text.split_at(pos);

//     // Do not trim aggressively; only normalize existing Windows-style newlines
//     let left_trimmed = left.strip_suffix("\r").unwrap_or(left); // Strip only if it's \r\n
//     let right_trimmed = right.strip_prefix("\n").unwrap_or(right); // Strip only if it's \n\r

//     let new_text = if last_key_was_enter {
//         // If the previous key was also Enter, insert two new lines to create a blank paragraph
//         format!("{}\r\n{}", left_trimmed, right_trimmed)
//     } else {
//         // Otherwise, just insert a single `\n\r`
//         format!("{}\r\n{}", left_trimmed, right_trimmed)
//     };

//     // Move caret position accordingly
//     let updated_pos = left_trimmed.len() + if last_key_was_enter { 2 } else { 2 };

//     (new_text, updated_pos)
// }

