// mod markdown;
mod state;
mod builder;
mod handler;
mod coroutines;
mod syntax;

use coroutines::update_editor_text_coroutine;
use dioxus::prelude::*;
use dioxus::events::Key;
use dioxus::logger::tracing::info;
use tokio;
use std::os::raw;
use std::time::Duration;

use dioxus::desktop::{Config, WindowBuilder, LogicalSize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, Level};

use base64::{engine::general_purpose, Engine as _};

use state::State;
use builder::EditorBuilder;
use std::collections::VecDeque;
use syntax::text::{MarkDownElements, TextProcessor};
use syntax::markdown::{compute_markdown_style_props, compute_markdown_style_string, CellInfo as MarkDownCellInfo};

// static FUNCTIONS_JS: Asset = asset!("/assets/functions.js");
fn main() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new()
            .with_window(WindowBuilder::new()
                .with_title("Probably Crater")    
                .with_inner_size(LogicalSize::new(490, 530))
                .with_min_inner_size(LogicalSize::new(600, 500))
                .with_resizable(true)
            ))
        .launch(App)
}

pub fn get_element_id(index_i: usize, index_j: usize) -> String {
    format!("textarea-{}-{}", index_i, index_j)
}


#[component]
fn App() -> Element {

    let input_text = "# Heading 1\n\tExample text for textarea1\nExample text for textarea 2 **with bold**".to_string();

    // let text_split = text::process_text(input_text);

    let mut editor = use_signal(|| EditorBuilder::new(
        None, // No UCI coroutine for now
        State::new(
            // vec![vec!["\tExample text for textarea 1".to_string(), "Example text for textarea 2".to_string()]],
            input_text.clone(),
            None,
        )
    ));

    let mut visual_editor = use_signal(|| State::new(
        // vec![vec!["\tExample text for textarea 1".to_string(), "Example text for textarea 2".to_string()]],
        input_text,
        None,
    ).raw_text);

    let mut is_mouse_down = use_signal(|| false);
    let mut is_mouse_dragging = use_signal(|| false);
    let mut last_cursor_position = use_signal(|| 0.0);
    

    let measure_width = use_coroutine(move |rx| async move {
        coroutines::measure_width_coroutine(rx, &mut editor).await;
    });

    // Swap names with focus_element
    let focus_caret_position = use_coroutine(move |rx: UnboundedReceiver<(usize, usize)>| async move {
        coroutines::focus_caret_position_coroutine(rx, &mut editor).await;
    });

    
    // Combined DOM updates queue for text and DOM operations
    let mut dom_updates = use_signal(|| VecDeque::<(String, String, Option<String>)>::new());


    // Function to focus an element by ID
    let focus_element = move |index_i: usize, index_j: usize, cursor_pos: usize| {

        let element_id = get_element_id(index_i, index_j);
        
        spawn(async move {
            // tokio::time::sleep(Duration::from_millis(1)).await;
            let js = format!(
                r#"
                return window.focusElementAndSetCaret('{}', {});
                "#,
                element_id,
                cursor_pos
            );
            let _ = document::eval(&js).await;

            // tokio::time::sleep(Duration::from_millis(1)).await;
            focus_caret_position.send((index_i, index_j));
            
        });
    };

    let mut internal_process = use_signal(|| false);

    let update_syntax = move || {
        
        async move {
            tokio::time::sleep(Duration::from_micros(100)).await;
            while (*dom_updates.read()).len() > 0 {
                println!("waiting for updates {}", (*dom_updates.read()).len());
                tokio::time::sleep(Duration::from_micros(20)).await;
            }

            

            let mut input_text = editor.read().raw_text.clone();
            
            // Remove empty elements from rows with more than one element
            for row in input_text.iter_mut() {
                if row.len() > 1 {
                    // Remove empty strings but keep at least one element
                    let non_empty: Vec<String> = row.iter().filter(|s| !s.is_empty()).cloned().collect();
                    if !non_empty.is_empty() {
                        *row = non_empty;
                    } else {
                        // If all were empty, keep one empty string
                        *row = vec![String::new()];
                    }
                }
            }
        
            // let editor_pos = get_editor_caret_position(index_i, index_j).await;

            let maybe_current_caret = editor.read().get_caret_pos();
            let row_level_caret_pos = editor.read().get_row_level_caret_pos(maybe_current_caret);
    
            // println!("{:?}", input_text);
            let text_processor = TextProcessor::new();
            let join_text = text_processor.markdown_to_string(input_text.clone());
            let syntax = text_processor.process_markdown(join_text.clone());
            let syntax_text = text_processor.extract_strings(syntax.clone());
            
            // Compare rows and identify changes
            let mut changed_rows = Vec::new();
            let max_rows = input_text.len().max(syntax_text.len());

            for row_idx in 0..max_rows {
                let input_row = input_text.get(row_idx);
                let syntax_row = syntax_text.get(row_idx);
                
                match (input_row, syntax_row) {
                    (Some(input), Some(syntax)) => {
                        // Both rows exist - compare them
                        if input != syntax {
                            changed_rows.push(row_idx);
                        }
                    }
                    (None, Some(_)) => {
                        // New row in syntax_text
                        changed_rows.push(row_idx);
                    }
                    (Some(_), None) => {
                        // Row removed in syntax_text
                        changed_rows.push(row_idx);
                    }
                    (None, None) => {
                        // Should not happen given our loop condition
                        break;
                    }
                }
            }

            // Early return if no changes
            if changed_rows.is_empty() {
                return;
            }
            
            

            // Process only changed rows
            if let Some((cur_index_i, cur_index_j, cur_caret_pos)) = maybe_current_caret {
                println!("[update syntax] index i {} index j {} cursor {}", cur_index_i, cur_index_j, cur_caret_pos);
                
                // Ensure text_width array is properly sized
                // while editor.read().text_width.len() <= cur_index_i {
                //     editor.write().text_width.push(vec![None]);
                // }
                
                // while editor.read().text_width[cur_index_i].len() <= cur_index_j {
                //     editor.write().text_width[cur_index_i].push(None);
                // }

                dom_updates.write().push_back(("internal_process".to_string(), "".to_string(), Some("true".to_string())));
                
                // Only update if current row changed
                if changed_rows.contains(&cur_index_i) && cur_index_i < syntax_text.len() {
                    // Safely get column index
                    let safe_cur_index_j = cur_index_j.min(syntax_text[cur_index_i].len().saturating_sub(1));
                    
                    if !syntax_text[cur_index_i].is_empty() && safe_cur_index_j < syntax_text[cur_index_i].len() {
                        let cur_text = syntax_text[cur_index_i][safe_cur_index_j].clone();
                        
                        println!("row {:?}", syntax_text[cur_index_i]);
                        println!("{:?}", safe_cur_index_j);
                        

                        // Update the editor's raw_text for this row
                        if cur_index_i < editor.read().raw_text.len() {
                            let old_col_count = editor.read().raw_text[cur_index_i].len();
                            let new_col_count = syntax_text[cur_index_i].len();
                            
                            // if old_col_count > new_col_count {
                            //     focus_element(cur_index_i, new_col_count - 1, editor.read().raw_text[cur_index_i][new_col_count - 1].len() - 1);
                            // }

                            // Clear columns that exceed the new column count
                            if old_col_count > new_col_count {
                                for col_idx in (new_col_count..old_col_count).rev() {
                                    // delete_element(cur_index_i, col_idx);
                                    let id = format!("{},{},{}", cur_index_i, col_idx, 0);
                                    dom_updates.write().push_back(("update_text".to_string(), id, Some("".to_string())));

        
                                }
                            }

                            //     // focus_element(cur_index_i, new_col_count - 1, editor.read().raw_text[cur_index_i][new_col_count - 1].len() - 1);
                            // }

                            // if old_col_count < new_col_count {
                            //     for col_idx in old_col_count..new_col_count {
                                    
                            //         let txt = syntax_text[cur_index_i][col_idx].clone();
                                    
                            //         let id = format!("{},{},{}", cur_index_i, col_idx, 0);
                            //         dom_updates.write().push_back(("update_text".to_string(), id, Some(txt)));
                            //     }
                            // }
                            

                                                        // Clear columns that exceed the new column count
                            // if old_col_count > new_col_count {
                            //     for col_idx in new_col_count..old_col_count {
                            //         // delete_element(cur_index_i, col_idx);
                            //         let id = format!("{},{},{}", cur_index_i, col_idx, 0);
                            //         dom_updates.write().push_back(("update_text".to_string(), id, Some("".to_string())));

                            //         let element_id = get_element_id(cur_index_i, col_idx);
                            //         println!("[delete element] {cur_index_i} {col_idx}");
                            //         let delete_payload = format!("{},{}", cur_index_i, col_idx);
                            //         dom_updates.write().push_back(("delete_element".to_string(), element_id, Some(delete_payload)));

                            //     }

                            //     // focus_element(cur_index_i, new_col_count - 1, editor.read().raw_text[cur_index_i][new_col_count - 1].len() - 1);
                            // }

                            
                            // // Create missing columns if new count is larger
                            // if old_col_count < new_col_count {
                            //     for col_idx in old_col_count..new_col_count {
                            //         let txt = syntax_text[cur_index_i][col_idx].clone();
                            //         let element_id = get_element_id(cur_index_i, col_idx);
                            //         let row_id = format!("textrow-{}", cur_index_i);
                            //         println!("[create element] {cur_index_i} {col_idx}");
                                    
                            //         // Format: "index_i,index_j,row_id,text_content,cell_style"
                            //         let cell_data = format!("{},{},{},{},'", cur_index_i, col_idx, row_id, txt);
                            //         dom_updates.write().push_back(("create_cell".to_string(), element_id, Some(cell_data)));

                            //         let txt = syntax_text[cur_index_i][col_idx].clone();
                                    
                            //         let id = format!("{},{},{}", cur_index_i, col_idx, 0);
                            //         dom_updates.write().push_back(("update_text".to_string(), id, Some(txt)));
                            //     }
                            // }
                            

                            // editor.write().raw_text[cur_index_i] = syntax_text[cur_index_i].clone();
                            let col_num = syntax_text[cur_index_i].len();
                            if col_num > 1 {
                                for col_indx in 0..col_num {
                                    let prev_text = syntax_text[cur_index_i][col_indx].clone();
                                    // Send to dom_updates instead of editor_updates
                                    let id = format!("{},{},{}", cur_index_i, col_indx, prev_text.len());

                                    
                                    dom_updates.write().push_back(("update_text".to_string(), id, Some(prev_text.clone())));
                                    
                                    measure_width.send((cur_index_i, col_indx, prev_text.clone()));
                            
                                    println!("current index text {} {} {:?} {}", cur_index_i, col_indx, prev_text, prev_text.len());
                                }
                            }
                        }
                        dom_updates.write().push_back(("internal_process".to_string(), "".to_string(), Some("false".to_string())));
                        // Send updates
                        // measure_width.send((cur_index_i, safe_cur_index_j, cur_text.to_string()));
                        
  
                        
                        // editor_updates.write().push_back((cur_index_i, safe_cur_index_j, cur_caret_pos, cur_text));
                        
                        // Handle caret positioning
                        if let Some(global_row_caret_pos) = row_level_caret_pos {
                            let maybe_new_caret_pos = editor.read().get_caret_from_row_level_pos(
                                global_row_caret_pos, 
                                cur_index_i, 
                                syntax_text[cur_index_i].clone()
                            );
                            
                            println!("new caret: {:?}", maybe_new_caret_pos);
                            if let Some((index_i, index_j, char_pos)) = maybe_new_caret_pos {
                                if index_i < syntax_text.len() && index_j < syntax_text[index_i].len() {
                                    let cur_text = syntax_text[index_i][index_j].clone();
                                    
                                    // editor.write().raw_text[index_i][index_j] = cur_text.clone();
                                    
                                    // Send to dom_updates instead of editor_updates
                                    let id = format!("{},{},{}", index_i, index_j, char_pos);
                                    dom_updates.write().push_back(("update_text_cursor".to_string(), id, Some(cur_text.to_string())));

                                    measure_width.send((index_i, index_j, cur_text.clone()));

                                    let id = format!("{},{},{}", index_i, index_j, char_pos);
                                    dom_updates.write().push_back(("focus_element".to_string(), id, Some("".to_string())));

                                    focus_element(index_i, index_j, char_pos);
                                }
                            }
                        }
                    }
                }
            }
    
            // while (*editor_updates.read()).len() > 0 {
            //     println!("waiting for updates {}", (*editor_updates.read()).len());
            //     tokio::time::sleep(Duration::from_micros(20)).await;
            // }

            for (index_i, inner) in editor.read().raw_text.iter().enumerate() {
                for (index_j, text) in inner.iter().enumerate() {
                    if text.len() > 0 {
                        measure_width.send((index_i, index_j, text.clone().to_string()));
                    }
                }
            }

            // visual_editor.set(editor.read().raw_text.clone());

            println!("input text: {:?}", input_text);
            println!("read text: {:?}", editor.read().raw_text);
            println!("raw text: {:?}", syntax_text);

            
        }
    };
    
    let update_editor_dom = use_coroutine(move |mut rx: UnboundedReceiver<(String, String, Option<String>)>| async move {
        loop {
            // Try to receive a message
            match rx.try_next() {
                Ok(Some((operation, id, data))) => {
            match operation.as_str() {
                "update_text" => {
                    // Handle text updates - id format: "index_i,index_j,pos", data: text
                    if let Some(text) = data {
                        let parts: Vec<&str> = id.split(',').collect();
                        if parts.len() == 3 {
                            if let (Ok(row), Ok(col), Ok(pos)) = (
                                parts[0].parse::<usize>(),
                                parts[1].parse::<usize>(),
                                parts[2].parse::<usize>()
                            ) {
                                let element_id = format!("textarea-{}-{}", row, col);
                                // Call the original text update coroutine logic
                                let js = format!(r#"return window.clearElementText('{}', '{}');"#, 
                                element_id, text.replace("'", "\\'"));
                                let _ = document::eval(&js).await;
                                println!("[update text] {} {}", element_id, text.clone());
                                editor.write().update_text(row, col, text.clone());
                            }
                        }
                    }
                }
                "update_text_cursor" => {
                    // Handle text updates - id format: "index_i,index_j,pos", data: text
                    if let Some(text) = data {
                        let parts: Vec<&str> = id.split(',').collect();
                        if parts.len() == 3 {
                            if let (Ok(index_i), Ok(index_j), Ok(pos)) = (
                                parts[0].parse::<usize>(),
                                parts[1].parse::<usize>(),
                                parts[2].parse::<usize>()
                            ) {
                                // Call the original text update coroutine logic
                                coroutines::update_editor_text_coroutine_single(index_i, index_j, pos, text, &mut editor).await;
                            }
                        }
                    }
                    println!("read text: {:?}", editor.read().raw_text);
                    println!("visual text: {:?}", visual_editor.read());
                }
                "delete_row" => {
                    if let Some(index_str) = data {
                        if let Ok(index_i) = index_str.parse::<usize>() {
                                
                                // First, call JavaScript to delete the row and renumber DOM
                                // let js = format!(r#"return window.deleteRow('{}');"#, id);
                                // let _ = document::eval(&js).await;
                                
                                // Remove from editor state after DOM update
                                editor.with_mut(|e| {
                                    if index_i < e.raw_text.len() {
                                        e.raw_text.remove(index_i);
                                    }
                                });
                                
                                visual_editor.with_mut(|raw_text| {
                                    if index_i < raw_text.len() {
                                        raw_text.remove(index_i);
                                    }
                                });
                                
                                editor.with_mut(|e| {
                                    for row in index_i..e.raw_text.len() {
                                        for col in 0..e.raw_text[row].len() {
                                            let text_content = e.raw_text[row][col].clone();
                                            let element_id = format!("textarea-{}-{}", row, col);
                                            spawn(async move {
                                                let js = format!(r#"return window.clearElementText('{}', '{}');"#, 
                                                    element_id, text_content.replace("'", "\\'"));
                                                let _ = document::eval(&js).await;
                                            });
                                        }
                                    }
                                });

                                // Parse and apply all text updates in spawned tasks
                                // if let Ok(row_updates) = serde_json::from_str::<Vec<(String, String)>>(updates_json) {
                                //     for (element_id, text_content) in row_updates {
                                //         spawn(async move {
                                //             let js = format!(r#"return window.updateElementText('{}', '{}');"#, 
                                //                 element_id, text_content.replace("'", "\\'"));
                                //             let _ = document::eval(&js).await;
                                //         });
                                //     }
                                // }
                                
                                // println!("read text: {:?}", editor.read().raw_text);
                                // println!("visual text: {:?}", visual_editor.read());
                        }
                    } else {
                        // Fallback for old format
                        let js = format!(r#"return window.deleteRow('{}');"#, id);
                        let _ = document::eval(&js).await;
                    }
                 }
                "internal_process" => {
                    if let Some(state) = data {
                        if let Ok(is_processing) = state.parse::<bool>() {
                            internal_process.set(is_processing);
                            println!("Internal process state set to: {}", is_processing);
                        }
                    }
                }
                "focus_element" => {
                    if let Some(focus_data) = data {
                        // Parse payload: "index_i,index_j,cursor_pos"
                        let parts: Vec<&str> = focus_data.split(',').collect();
                        if parts.len() == 3 {
                            if let (Ok(index_i), Ok(index_j), Ok(cursor_pos)) = (
                                parts[0].parse::<usize>(),
                                parts[1].parse::<usize>(),
                                parts[2].parse::<usize>()
                            ) {
                                let element_id = get_element_id(index_i, index_j);
                                
                                spawn(async move {
                                    let js = format!(
                                        r#"
                                        return window.focusElementAndSetCaret('{}', {});
                                        "#,
                                        element_id.clone(),
                                        cursor_pos
                                    );
                                    let _ = document::eval(&js).await;
                                });
                                
                                focus_caret_position.send((index_i, index_j));
                                println!("Focused element: ({}, {}) at position {}", index_i, index_j, cursor_pos);
                            }
                        }
                    }
                }
                    
                "create_row" => {
                    if let Some(create_data) = data {
                        // Parse the create_row_data: "index_i|index_j|new_row_json"
                        let parts: Vec<&str> = create_data.split('|').collect();
                        if parts.len() == 3 {
                            if let (Ok(index_i), Ok(index_j)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                                let new_row_json = parts[2];
                                
                                // Parse the new_row data
                                if let Ok(new_row) = serde_json::from_str::<Vec<String>>(new_row_json) {
                                    // First, call JavaScript to create DOM row and renumber
                                    // let js = format!(r#"return window.createRow('{}', 'textrow-{}');"#, id, index_i);
                                    // let _ = document::eval(&js).await;
                                    
                                    // Then, update editor state after DOM update
                                    editor.with_mut(|e| {
                                        // Insert the new row
                                        e.raw_text.insert(index_i + 1, new_row.clone());
                                        // Move caret to beginning of new row
                                        e.move_caret(index_i + 1, 0, 0);
                                    });

                                    visual_editor.with_mut(|raw_text| {
                                        raw_text.insert(index_i + 1, new_row.clone());
                                    });
                                    
                                    editor.with_mut(|e| {
                                        for row in (index_i + 1)..e.raw_text.len() {
                                            for col in 0..e.raw_text[row].len() {
                                                let text_content = e.raw_text[row][col].clone();
                                                let element_id = format!("textarea-{}-{}", row, col);
                                                spawn(async move {
                                                    let js = format!(r#"return window.clearElementText('{}', '{}');"#, 
                                                        element_id, text_content.replace("'", "\\'"));
                                                    let _ = document::eval(&js).await;
                                                });
                                            }
                                        }
                                    });

                                    // The text updates for the new row cells will be handled 
                                    // by the dom_updates queue from the handler

                                                                
                                }

                                
                            }
                        }
                    } else {
                        // Fallback for old format
                        let js = format!(r#"return window.createRow('{}');"#, id);
                        let _ = document::eval(&js).await;
                    }
                }
                "delete_element" => {
                    let js = format!(r#"return window.deleteElement('{}');"#, id);
                    let _ = document::eval(&js).await;

                    if let Some(payload) = data {
                        // Parse payload: "index_i,index_j"
                        let parts: Vec<&str> = payload.split(',').collect();
                        if parts.len() == 2 {
                            if let (Ok(index_i), Ok(index_j)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                                // Remove the specific element from raw_text
                                editor.with_mut(|e| {
                                    if index_i < e.raw_text.len() && index_j < e.raw_text[index_i].len() {
                                        e.raw_text[index_i].remove(index_j);
                                    }
                                });
                                
                                visual_editor.with_mut(|raw_text| {
                                    if index_i < raw_text.len() && index_j < raw_text[index_i].len() {
                                        raw_text[index_i].remove(index_j);
                                    }
                                });
                            }
                        }
                    }
                }
                "create_cell" => {
                    if let Some(cell_data) = data {
                        // Format: "index_i,index_j,row_id,text_content,cell_style"
                        let parts: Vec<&str> = cell_data.split(',').collect();
                        if parts.len() >= 5 {
                            if let (Ok(index_i), Ok(index_j)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                                let row_id = parts[2];
                                let text_content = parts[3];
                                let cell_style = parts[4];
                                
                                // Insert text at index_i, index_j in editor state
                                editor.with_mut(|e| {
                                    // Ensure the row exists
                                    while e.raw_text.len() <= index_i {
                                        e.raw_text.push(vec![String::new()]);
                                    }
                                    // Ensure the column exists in this row
                                    while e.raw_text[index_i].len() <= index_j {
                                        e.raw_text[index_i].push(String::new());
                                    }
                                    // Set the text content
                                    e.raw_text[index_i][index_j] = text_content.to_string();
                                });

                                visual_editor.with_mut(|raw_text| {
                                    // Ensure the row exists
                                    while raw_text.len() <= index_i {
                                        raw_text.push(vec![String::new()]);
                                    }
                                    // Ensure the column exists in this row
                                    while raw_text[index_i].len() <= index_j {
                                        raw_text[index_i].push(String::new());
                                    }
                                    // Set the text content
                                    raw_text[index_i][index_j] = text_content.to_string();
                                });
                                
                                let js = format!(r#"return window.createCell('{}', '{}', '{}', '{}');"#, 
                                    id, row_id, text_content, cell_style);
                                let _ = document::eval(&js).await;

                                println!("create raw_text: {:?}", editor.read().raw_text);
                            }
                        }
                    }
                }
                "update_row" => {
                    if let Some(row_data) = data {
                        // Parse payload: "index_i|row_data_json"
                        let parts: Vec<&str> = row_data.split('|').collect();
                        if parts.len() == 2 {
                            if let Ok(index_i) = parts[0].parse::<usize>() {
                                // let row_data_json = parts[1];

                                let current_row = editor.read().raw_text[index_i+1].clone();
                                let col = editor.read().raw_text[index_i].len() - 1;
                                let cursor_pos = editor.read().raw_text[index_i][col].len();

                                editor.with_mut(|e| {
                                    // Insert the new row

                
                                    // Add all columns after the current one to the new row
                                    for col_idx in 0..current_row.len() {
                                        if col_idx == 0 {
                                            e.raw_text[index_i][col].push_str(current_row[col_idx].as_str());
                                        } else {
                                            e.raw_text[index_i].push(current_row[col_idx].clone());
                                        }
                                    }
                                    // Move caret to beginning of new row
                                    e.move_caret(index_i, col, cursor_pos);
                                });

                                visual_editor.with_mut(|raw_text| {
                                    // let mut new_row = raw_text[index_i].clone();
                                    // Add all columns after the current one to the new row
                                    for col_idx in 0..current_row.len() {
                                        if col_idx == 0 {
                                            raw_text[index_i][col].push_str(current_row[col_idx].as_str());
                                        } else {
                                            raw_text[index_i].push(current_row[col_idx].clone());
                                        }
                                    }
                                });

                                editor.with_mut(|e| {
                                let row = index_i;
                                for col in 0..e.raw_text[row].len() {
                                    let text_content = e.raw_text[row][col].clone();
                                    let element_id = format!("textarea-{}-{}", row, col);
                                    spawn(async move {
                                        let js = format!(r#"return window.clearElementText('{}', '{}');"#, 
                                            element_id, text_content.replace("'", "\\'"));
                                        let _ = document::eval(&js).await;
                                    });
                                }
                            
                                });
                                editor.with_mut(|e| {
                                    // Insert the new row

                                    e.move_caret(index_i, col, cursor_pos);
                                });

                                focus_element(index_i, col, cursor_pos);
                               
                            }
                        }
                    }
                }
                _ => {
                    println!("Unknown DOM operation: {}", operation);
                }
            };

            if *internal_process.read() == false {
                let _ = update_syntax().await;
                visual_editor.set(editor.read().raw_text.clone());
            } 
            // else {

            
            // spawn( async move {

                
            //         tokio::time::sleep(Duration::from_micros(1)).await;

            // });
            // }
        }
        _ => {
            tokio::time::sleep(Duration::from_micros(2)).await;
        }
        }
    
        
    }
    println!("update dom ended!!!!!!");
    });


    // let update_editor_text = move |index_i: usize, index_j: usize, cursor_pos: usize, new_text: String| {
        
    //     spawn(async move {
    //         tokio::time::sleep(Duration::from_millis(5)).await;
    //         let element_id = get_element_id(index_i, index_j);
    //         let text_b64 = general_purpose::STANDARD.encode(new_text.clone());

    //         let js = format!(
    //             r#"
    //             return window.clearElementText('{}', atob("{}"), {});
    //             "#,
    //             element_id,
    //             text_b64,
    //             cursor_pos
    //         );

    //         let _ = document::eval(&js).await;
            
    //         editor.write().update_text(index_i, index_j, new_text.clone());

    //         editor.write().move_caret(index_i, index_j , cursor_pos);
    //     });
    // };

    let _sync_task = use_coroutine(move |rx: UnboundedReceiver<()>| async move {
        // let mut files_container = files_container.clone(); // Clone the signal
        loop {
            tokio::time::sleep(Duration::from_micros(1)).await;
            if (*dom_updates.read()).len() > 0 {
                
                if let Some((operation, id, data)) = (*dom_updates.write()).pop_front() {        
   
                    // Process the DOM update
                    update_editor_dom.send((operation, id, data));

                    
                }


            }

            // println!("Updated files: {:?}", files_ref.md_file_paths);
        }
    });


    async fn get_editor_text(index_i: usize, index_j: usize) -> Option<String> {
        let element_id = get_element_id(index_i, index_j);
        // JavaScript to get the caret position within the element
        let js_code = format!(
            r#"
            return window.getElementText('{}');
            "#,
            element_id
        );

        match document::eval(&js_code).await {
            Ok(result) => {
                if let Some(text) = result.as_str() {
                    let text = text.to_string();
                    Some(text)
                    // println!("Caret position for index {} {}: {}", index_i, index_j, pos);
                    
                } else {
                    eprintln!("Failed to parse caret position: {:?}", result);
                    return None
                }
            }
            Err(e) => {
                eprintln!("JS eval error: {}", e);
                return None
            }
            
        }
    };

    async fn get_editor_caret_position(index_i: usize, index_j: usize) -> Option<usize> {
        let element_id = get_element_id(index_i, index_j);
        // JavaScript to get the caret position within the element
        let js_code = format!(
            r#"
            return window.getCaretClickPosition('{}');
            "#,
            element_id
        );

        match document::eval(&js_code).await {
            Ok(result) => {
                if let Some(pos) = result.as_i64() {
                    let pos = pos as usize;
                    Some(pos)
                    // println!("Caret position for index {} {}: {}", index_i, index_j, pos);
                    
                } else {
                    eprintln!("Failed to parse caret position: {:?}", result);
                    return None
                }
            }
            Err(e) => {
                eprintln!("JS eval error: {}", e);
                return None
            }
            
        }
    };



    let delete_element = move |index_i: usize, index_j: usize| {
        let element_id = get_element_id(index_i, index_j);
        println!("[delete element] {index_i} {index_j}");
        update_editor_dom.send(("delete_element".to_string(), element_id, None));
    };

    let delete_row = move |index_i: usize| {
        let row_id = format!("textrow-{}", index_i);
        println!("[delete row  ] {index_i}");
        update_editor_dom.send(("delete_row".to_string(), row_id, None));
    };

    // use_effect(move || {
    //     for (index_i, inner) in editor.read().raw_text.iter().enumerate() {
    //         for (index_j, text) in inner.iter().enumerate() {
    //             if text.len() > 0 {
    //                 measure_width.send((index_i, index_j, text.clone().to_string()));
    //             }
    //         }
    //     }
    // });

    let editor_js = use_future(|| async move {

        let js_code = include_str!("../assets/editor.js");
        // println!("{}", js_code);
        if let Err(e) = document::eval(&js_code).await {
            println!("JS eval error: {:?}", e); // Add this line
        } else if let Ok(result) = document::eval(&js_code).await {
            println!("result {:?}", result);
            
        }
    }
    );

    let handle_keydown_input = move |event: KeyboardEvent, index_i: usize, index_j: usize| {
    
    
        let element_id = format!("textarea-{}-{}", index_i, index_j);
    


        let cur_text = editor.read().get_raw_text_current();

        if event.key() == Key::Enter {
            event.stop_propagation();
            event.prevent_default();

            let _ = handler::handle_enter_key(event, index_i, index_j, &editor, measure_width, dom_updates);

        } else if event.key() == Key::ArrowLeft {

            println!("[left arrow] current text {}, len {}", cur_text, cur_text.len());

            let _ = handler::handle_left_arrow(event, index_i, index_j, &editor, focus_element, focus_caret_position, get_editor_caret_position);

            // println!("[left arrow] current indx {}", indx_j);
        } else if event.key() == Key::ArrowRight {

            let _ = handler::handle_right_arrow(event, index_i, index_j, &editor, focus_element, focus_caret_position, get_editor_caret_position);
        } else if event.key() == Key::ArrowUp {

            let _ = handler::handle_up_arrow(event, index_i, index_j, &editor, focus_element, focus_caret_position, get_editor_caret_position);
        } else if event.key() == Key::ArrowDown {

            let _ = handler::handle_down_arrow(event, index_i, index_j, &editor, focus_element, focus_caret_position, get_editor_caret_position);
        } else if event.key() == Key::Backspace {

            let _ = handler::handle_backspace(event, index_i, index_j, &editor, dom_updates, update_editor_dom, focus_element, 
                delete_row,
                get_editor_caret_position, get_editor_text);
        } else {
            let _ = handler::handle_character_input(event, index_i, index_j, &editor, dom_updates, update_editor_dom, get_editor_caret_position, get_editor_text);
        }

    };
    
    // let renderer = use_memo(move || {
    //     editor.read().raw_text.clone()
    // });
   

    let renderer = use_memo(move || {


        let input_text = visual_editor.read().clone();
        
        let maybe_current_caret = editor.read().get_caret_pos();
        let row_level_caret_pos = editor.read().get_row_level_caret_pos(maybe_current_caret);

        // println!("{:?}", input_text);
        let text_processor = TextProcessor::new();
        let join_text = text_processor.markdown_to_string(input_text.clone());
        let syntax_text = text_processor.process_markdown(join_text.clone());
        let raw_text = text_processor.extract_strings(syntax_text.clone());
        // raw_text
        // input_text
        syntax_text
    });


    let iter_format_cell = move |row: usize, col: usize, num_cols: usize, text: MarkDownElements| {

        let flat_index = editor.read().raw_text.iter().take(row).map(|inner| inner.len()).sum::<usize>() + col;
        
        
        // let width = editor.read().text_width[flat_index].map_or("auto".to_string(), |w| format!("{}px", w));
        
        let width = if row > editor.read().text_width.len() - 1 {
            None
        }
        else if col > editor.read().text_width[row].len() - 1 {
            None
        }
        else {
            editor.read().text_width[row][col]
        };
        

        // let text = (*text).clone();
        let element_id = format!("textarea-{}-{}", row, col);

        let attrs = compute_markdown_style_props(
            MarkDownCellInfo {
                row: row,
                col: col,
                num_cols: num_cols,
                width: width,
                syntax: text,
            }
        );

        let text = attrs.text.clone();

        let cell_style = compute_markdown_style_string(attrs); 

        rsx! {
            div {
                id: element_id.clone(),
                contenteditable: !(is_mouse_dragging.read().clone()),
                class: "base-paragraph",
                style: cell_style,

                onkeydown: move |event| {
                    if (*dom_updates.read()).len() < 20 { 
                        handle_keydown_input(event, row, col);
                    }
                    
                },
                onmousedown: move |event| { is_mouse_down.set(true) },
                onmousemove: move |event| {
                    if *is_mouse_down.read() {
                        is_mouse_dragging.set(true);
                    }
                },
                onmouseup: move |event| {
                    is_mouse_down.set(false);
                    is_mouse_dragging.set(false);
                },
                onclick: move |_| {
                    focus_caret_position.send((row, col));
                },
                if text.len() > 0 {
                    "{text}"
                }
                else  {
                    ""
                }
            }
        }
    };

    rsx! {
        document::Link { href: asset!("/assets/editor.css"), rel: "stylesheet"}
        div {
            style: "display: flex; flex-direction: column;",
            id: "container",
            {
                renderer().iter().enumerate().map(|(row, inner)| {
                    // println!("rerendered");
                    rsx! {
                        div {
                            style: "display: flex; flex-direction: row; gap: 0; flex-wrap: wrap; font-size: 0;",
                            id: "textrow-{row}",
                            {
                                inner.iter().enumerate().map(move |(col, text)| {
                                    // Compute flat index for text_width
                
                                    let num_cols =inner.len();
                                    {
                                        iter_format_cell(row, col, num_cols, text.clone())
                                    }
                                })
                            }
                        }
                    }
                })
            }
        }
    }
}
#[derive(Debug)]
struct CellInfo {
    pub row: usize,
    pub col: usize,
    pub num_cols: usize,
    pub width: Option<f64>
}

pub fn compute_style(props: CellInfo) -> String {

    let width = props.width.map_or("auto".to_string(), |w| format!("{}px", w));

    if (props.col == props.num_cols - 1) {
        if props.row == 0 {
    
            format!("flex-grow: 1; font-weight: normal; width: {};", width)
        }
        else {

            format!("flex-grow: 1; font-weight: bold; width: {};", width)
        }
    } else if props.row == 0 {
        format!("flex-grow: 0; width: {};", width)
    } else {
        format!("flex-grow: 0; font-weight: bold; width: {};", width)
    }
}
