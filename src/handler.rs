use dioxus::prelude::*;
use crate::EditorBuilder;
use std::collections::VecDeque;
use std::f32::consts::E;
use std::future::Future;
use std::time::Duration;

use crate::get_element_id;

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
}

// Handler for Enter key
pub fn handle_enter_key(
    event: KeyboardEvent,
    index_i: usize,
    index_j: usize,
    editor: &Signal<EditorBuilder>,
    measure_width: Coroutine<(usize, usize, String)>,
    dom_updates: Signal<VecDeque<(String, String, Option<String>)>>,
) -> Result<(), &'static str>{
    event.stop_propagation();
    event.prevent_default();
    
    let mut editor = editor.clone();
    let mut dom_updates = dom_updates.clone();
    spawn(async move {
        editor.with_mut(|e| {
            let (caret_i, caret_j, caret_pos) = e.get_caret_pos().unwrap_or((index_i, index_j, 0));
            if caret_i == index_i && caret_j == index_j {
                let current_row = e.raw_text[index_i].clone();
                let current_text = current_row[index_j].clone();
                let (before, after) = current_text.split_at(caret_pos);

                // Update the current cell with the 'before' text
                e.update_text(index_i, index_j, before.to_string());
                
                // Create a new row with the 'after' text and all subsequent columns
                let mut new_row = vec![after.to_string()];
                
                // Add all columns after the current one to the new row
                for col_idx in (index_j + 1)..current_row.len() {
                    new_row.push(current_row[col_idx].clone());
                }
                println!("current row {:?}", current_row);
                println!("new row {:?}", new_row);
                // Clear the columns after the current one in the current row
                e.raw_text[index_i].truncate(index_j + 1);
                
                // Insert the new row
                e.raw_text.insert(index_i + 1, new_row.clone());
                
                measure_width.send((index_i, index_j, before.to_string()));
                measure_width.send((index_i + 1, 0, after.to_string()));

                // Move caret to beginning of new row
                e.move_caret(index_i + 1, 0, 0);

                let id1 = format!("{},{},{}", index_i, index_j, before.len());

                // DOM updates: update current cell, insert new row, then update ALL cells in new row
                // e.update_text(index_i, index_j, before.to_string());
                // e.move_caret(index_i, index_j , before.len());

                // e.update_text(index_i + 1, 0 , after.to_string());
                // e.move_caret(index_i + 1, 0 , after.len());

                dom_updates.write().push_back(("update_text".to_string(), id1, Some(before.to_string())));
                dom_updates.write().push_back(("create_row".to_string(), format!("textrow-{}", index_i + 1), Some(format!("textrow-{}", index_i))));
                
                
                // Update all columns in the new row
                for (new_col_idx, col_content) in new_row.iter().enumerate() {
                    let id = format!("{},{},{}", index_i + 1, new_col_idx, 0);
                    dom_updates.write().push_back(("update_text".to_string(), id, Some(col_content.clone())));
                }
                let id = format!("{},{},{}", index_i + 1, 0, 0);
                dom_updates.write().push_back(("update_text".to_string(), id, Some(after.to_string())));
            }
        });
    });

    Ok(())
}

pub fn handle_left_arrow<F, Fut>(
    event: KeyboardEvent,
    index_i: usize,
    index_j: usize,
    editor: &Signal<EditorBuilder>,
    focus_element: impl Fn(usize, usize, usize) + 'static,
    focus_caret_position: Coroutine<(usize, usize)>,
    get_editor_caret_position: F,
) -> Result<(), &'static str>
where
    F: Fn(usize, usize) -> Fut + 'static,
    Fut: std::future::Future<Output = Option<usize>> + 'static,
    {
    
    let mut editor = editor.clone();
    spawn(async move {
                

        let (current_index_i, current_index_j, current_caret_pos) = editor.read().get_caret_pos().unwrap_or((index_i, index_j, 0));

        let editor_pos = get_editor_caret_position(current_index_i, current_index_j).await;

        if (current_index_i == index_i) && (current_index_j == index_j) {
            
            if let Some(e_pos) = editor_pos {


            editor.with_mut(|e| {
                
                let (new_index_i, new_index_j, new_pos) = if (e_pos == 0) && (index_j > 0) {
                    // If the is a previous cell in the same row 
                    (index_i, index_j - 1, e.raw_text[index_i][index_j - 1].len().saturating_sub(1))
                } else if (e_pos == 0) && (index_j == 0) && (index_i > 0){
                    // If there is a previous row
                    let prev_i = index_i - 1;
                    let prev_j = e.raw_text[prev_i].len().saturating_sub(1);
                    (prev_i, prev_j, e.raw_text[prev_i][prev_j].len())
                } else if e_pos > 0 {
                    (index_i, index_j, e_pos - 1)
                }
                else {
                    (index_i, 0, 0)
                };

                if (new_index_j != index_j) || (new_index_i != index_i){
                    focus_element(new_index_i, new_index_j, new_pos);
                } else {
                    focus_caret_position.send((new_index_i, new_index_j));
                }

            });
        }

        }
    });

    Ok(())
}

pub fn handle_right_arrow<F, Fut>(
    event: KeyboardEvent,
    index_i: usize,
    index_j: usize,
    editor: &Signal<EditorBuilder>,
    focus_element: impl Fn(usize, usize, usize) + 'static,
    focus_caret_position: Coroutine<(usize, usize)>,
    get_editor_caret_position: F,
) -> Result<(), &'static str>
where
    F: Fn(usize, usize) -> Fut + 'static,
    Fut: std::future::Future<Output = Option<usize>> + 'static,
{
    event.stop_propagation();
    event.prevent_default();

    let mut editor = editor.clone();
    spawn(async move {
        let (current_index_i, current_index_j, current_caret_pos) = editor.read().get_caret_pos().unwrap_or((index_i, index_j, 0));

        let editor_pos = get_editor_caret_position(current_index_i, current_index_j).await;

        if (current_index_i == index_i) && (current_index_j == index_j) {
            
            if let Some(e_pos) = editor_pos {

                editor.with_mut(|e| e.move_caret(index_i, index_j, e_pos));

                let current_text_len = editor.read().get_raw_text_current().len().saturating_sub(1);
                let current_col_size = editor.read().raw_text[index_i].len().saturating_sub(1);
                let current_row_size = editor.read().raw_text.len().saturating_sub(1);
                println!("[handle keydown] {} {} {}", e_pos, current_caret_pos, current_text_len);
                
                editor.with_mut(|e| {
                    
                    let (new_index_i, new_index_j, new_pos) = if (current_caret_pos > current_text_len) && (index_j < current_col_size) {
                        // If there is a next cell in the same row 
                        (index_i, index_j + 1, 0)

                    } else if (current_caret_pos > current_text_len) && (index_j == current_col_size) && (index_i < current_row_size){
                        // If there is a next row
                        let next_i = index_i + 1;
                        let next_j = 0;
                        (next_i, next_j, 0)
                    } else if current_caret_pos <= current_text_len + 1 {
                        (index_i, index_j, current_caret_pos + 1)
                    }
                    else {
                        (current_row_size, current_col_size, current_text_len)
                    };

                    if (new_index_j != index_j) || (new_index_i != index_i){
                        focus_element(new_index_i, new_index_j, new_pos);
                    } else {
                        focus_caret_position.send((new_index_i, new_index_j));
                        focus_element(new_index_i, new_index_j, new_pos);
                    }

                });
            }
        }
    });

    Ok(())
}

pub fn handle_up_arrow<F, Fut>(
    event: KeyboardEvent,
    index_i: usize,
    index_j: usize,
    editor: &Signal<EditorBuilder>,
    focus_element: impl Fn(usize, usize, usize) + 'static,
    focus_caret_position: Coroutine<(usize, usize)>,
    get_editor_caret_position: F,
) -> Result<(), &'static str>
where
    F: Fn(usize, usize) -> Fut + 'static,
    Fut: std::future::Future<Output = Option<usize>> + 'static,
{
    event.stop_propagation();
    event.prevent_default();
    
    let mut editor = editor.clone();
    spawn(async move {
        let (current_index_i, current_index_j, current_caret_pos) = editor.read().get_caret_pos().unwrap_or((index_i, index_j, 0));

        let editor_pos = get_editor_caret_position(current_index_i, current_index_j).await;

        if (current_index_i == index_i) && (current_index_j == index_j) {
            
            if let Some(e_pos) = editor_pos {

                editor.with_mut(|e| {
                    
                    let (new_index_i, new_index_j, new_pos) = if index_i > 0 {
                        // Move to the previous row, same column position
                        let prev_i = index_i - 1;
                        let prev_j = if index_j < e.raw_text[prev_i].len() {
                            index_j
                        } else {
                            e.raw_text[prev_i].len().saturating_sub(1)
                        };
                        
                        // Try to maintain similar character position within the cell
                        let target_text_len = e.raw_text[prev_i][prev_j].len();
                        let new_char_pos = if e_pos < target_text_len {
                            e_pos
                        } else {
                            target_text_len
                        };
                        
                        (prev_i, prev_j, new_char_pos)
                    } else {
                        // Already at the top row, stay at current position
                        (index_i, index_j, e_pos)
                    };

                    if (new_index_j != index_j) || (new_index_i != index_i) {
                        focus_element(new_index_i, new_index_j, new_pos);
                    } else {
                        focus_caret_position.send((new_index_i, new_index_j));
                    }
                });
            }
        }
    });

    Ok(())
}

pub fn handle_down_arrow<F, Fut>(
    event: KeyboardEvent,
    index_i: usize,
    index_j: usize,
    editor: &Signal<EditorBuilder>,
    focus_element: impl Fn(usize, usize, usize) + 'static,
    focus_caret_position: Coroutine<(usize, usize)>,
    get_editor_caret_position: F,
) -> Result<(), &'static str>
where
    F: Fn(usize, usize) -> Fut + 'static,
    Fut: std::future::Future<Output = Option<usize>> + 'static,
{
    event.stop_propagation();
    event.prevent_default();
    
    let mut editor = editor.clone();
    spawn(async move {
        let (current_index_i, current_index_j, current_caret_pos) = editor.read().get_caret_pos().unwrap_or((index_i, index_j, 0));

        let editor_pos = get_editor_caret_position(current_index_i, current_index_j).await;

        if (current_index_i == index_i) && (current_index_j == index_j) {
            
            if let Some(e_pos) = editor_pos {

                editor.with_mut(|e| {
                    
                    let (new_index_i, new_index_j, new_pos) = if index_i < e.raw_text.len().saturating_sub(1) {
                        // Move to the next row, same column position
                        let next_i = index_i + 1;
                        let next_j = if index_j < e.raw_text[next_i].len() {
                            index_j
                        } else {
                            e.raw_text[next_i].len().saturating_sub(1)
                        };
                        
                        // Try to maintain similar character position within the cell
                        let target_text_len = e.raw_text[next_i][next_j].len();
                        let new_char_pos = if e_pos < target_text_len {
                            e_pos
                        } else {
                            target_text_len
                        };
                        
                        (next_i, next_j, new_char_pos)
                    } else {
                        // Already at the bottom row, stay at current position
                        (index_i, index_j, e_pos)
                    };

                    if (new_index_j != index_j) || (new_index_i != index_i) {
                        focus_element(new_index_i, new_index_j, new_pos);
                    } else {
                        focus_caret_position.send((new_index_i, new_index_j));
                    }
                });
            }
        }
    });

    Ok(())
}

pub fn handle_backspace<F1, Fut1, F2, Fut2>(
    event: KeyboardEvent,
    index_i: usize,
    index_j: usize,
    editor: &Signal<EditorBuilder>,
    dom_updates: Signal<VecDeque<(String, String, Option<String>)>>,
    update_editor_dom: Coroutine<(String, String, Option<String>)>,
    focus_element: impl Fn(usize, usize, usize) + 'static,
    delete_row: impl Fn(usize) + 'static,
    get_editor_caret_position: F1,
    get_editor_text: F2,
) -> Result<(), &'static str>
where
    F1: Fn(usize, usize) -> Fut1 + 'static,
    Fut1: std::future::Future<Output = Option<usize>> + 'static,
    F2: Fn(usize, usize) -> Fut2 + 'static,
    Fut2: std::future::Future<Output = Option<String>> + 'static,
{
    event.stop_propagation();
    event.prevent_default();
    
    let mut editor = editor.clone();
    let mut dom_updates = dom_updates.clone();

    println!("editor {:?}", editor.read().raw_text);
    
    spawn(async move {

        while (*dom_updates.read()).len() > 0 {
            println!("waiting for updates {}", (*dom_updates.read()).len());
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        let (current_index_i, current_index_j, current_caret_pos) = editor.read().get_caret_pos().unwrap_or((index_i, index_j, 0));
        
        // Get current text from editor.raw_text with bounds checking
        let current_text = if current_index_i < editor.read().raw_text.len() && current_index_j < editor.read().raw_text[current_index_i].len() {
            editor.read().raw_text[current_index_i][current_index_j].clone()
        } else {
            String::new()
        };

        let e_pos = current_caret_pos;
        let e_text = current_text;
        
        // Check if cursor is at beginning of a cell
        if e_pos == 0 {
            
            if index_i > 0 {
                // Move row up: merge current row with previous row
                
                editor.with_mut(|e| {
                    let prev_i = index_i - 1;
                    let current_row = e.raw_text[index_i].clone();
                    println!("[pos=0 and i>0 and j==0 and key event]");
                    if index_j == 0 {
                        // When at the first column, merge entire row and clear the text that was moved
                        // Determine where to place the cursor: last column if previous row has multiple columns
                        let cursor_col_idx = if e.raw_text[prev_i].len() > 1 {
                            // Move to last column if previous row has multiple columns
                            e.raw_text[prev_i].len() - 1
                        } else {
                            // Stay at first column if previous row has only one column
                            0
                        };
                        
                        let prev_content_len = if cursor_col_idx < e.raw_text[prev_i].len() {
                            e.raw_text[prev_i][cursor_col_idx].len()
                        } else {
                            0
                        };
                        
                        

                        // Merge all columns from current row into previous row
                        for (col_idx, col_content) in current_row.iter().enumerate() {
                            e.raw_text[prev_i][cursor_col_idx].push_str(col_content)

                        }

                        let cur_text = e.raw_text[prev_i][cursor_col_idx].to_string();
                        // let tmp_cursor_pos = 0;
                        // e.update_text(index_i, index_j, cur_text.clone());
                        // e.move_caret(index_i, index_j , tmp_cursor_pos);

                        let id = format!("{},{},{}", prev_i, 0, prev_content_len);
                        dom_updates.write().push_back(("update_text".to_string(), id, Some(cur_text)));
                        
                        // Remove current row by emptying it
                        e.raw_text.remove(index_i);
                        
                        dom_updates.write().push_back(("delete_row".to_string(), format!("textrow-{}", index_i), None));
                        
                        // Focus on the cursor column at the merge point
                        // focus_element(prev_i, cursor_col_idx, prev_content_len);
                    } else {
                        println!("[pos=0 and i>0 and j>0 key event]");
                        // When not at the first column, just clear the current cell and move cursor
                        let tmp_cursor_pos = 0;
                        e.update_text(index_i, index_j, "".to_string());
                        e.move_caret(index_i, index_j , 0);

                        let id = format!("{},{},{}", index_i, index_j, 0);
                        dom_updates.write().push_back(("update_text".to_string(), id, Some("".to_string())));
                        // Move to the previous column
                        let prev_j = index_j - 1;
                        let prev_col_len = e.raw_text[index_i][prev_j].len();
                        
                        focus_element(index_i, prev_j, prev_col_len);
                    }
                });
                
            } else if index_j > 0 {
                // Move to previous column and delete a character
                println!("[else pos=0 and j>0 key event]");
                editor.with_mut(|e| {
                    let prev_j = index_j - 1;
                    
                    let tmp_cursor_pos = 0;
                    e.update_text(index_i, index_j, "".to_string());
                    e.move_caret(index_i, index_j , 0);

                    let id = format!("{},{},{}", index_i, index_j, 0);
                    dom_updates.write().push_back(("update_text".to_string(), id, Some("".to_string())));
                    
                    // Get the new cursor position (end of previous column)
                    let new_pos = e.raw_text[index_i][prev_j].len();
                    
                    // Focus on the previous column at the end
                    focus_element(index_i, prev_j, new_pos);
                });
            } else {
                println!("[else pos=0 key event]");
                // Normal text sync for regular backspace
            }
        } else {
            println!("default");
            // Normal backspace: delete character at cursor position
            if e_pos > 0 && e_pos <= e_text.len() {
                let mut cur_text = e_text.clone();
                cur_text.remove(e_pos - 1);
                let new_caret_pos = e_pos - 1;
                println!("new_caret_pos {:?}", new_caret_pos);
                // Update editor state
                editor.with_mut(|e| {
                    if current_index_i < e.raw_text.len() && current_index_j < e.raw_text[current_index_i].len() {
                        e.raw_text[current_index_i][current_index_j] = cur_text.clone();
                        e.move_caret(current_index_i, current_index_j, new_caret_pos);
                    }
                });
                
                // let tmp_cursor_pos = cur_text.len();
                editor.write().update_text(current_index_i, current_index_j, cur_text.clone());
                editor.write().move_caret(current_index_i, current_index_j , new_caret_pos);

                let id = format!("{},{},{}", current_index_i, current_index_j, new_caret_pos);
                dom_updates.write().push_back(("update_text".to_string(), id, Some(cur_text)));
            }
        }
    });
    Ok(())
}

pub fn handle_character_input<F1, Fut1, F2, Fut2>(
    event: KeyboardEvent,
    index_i: usize,
    index_j: usize,
    editor: &Signal<EditorBuilder>,
    dom_updates: Signal<VecDeque<(String, String, Option<String>)>>,
    update_editor_dom: Coroutine<(String, String, Option<String>)>,
    get_editor_caret_position: F1,
    get_editor_text: F2,
) -> Result<(), &'static str>
where
    F1: Fn(usize, usize) -> Fut1 + 'static,
    Fut1: std::future::Future<Output = Option<usize>> + 'static,
    F2: Fn(usize, usize) -> Fut2 + 'static,
    Fut2: std::future::Future<Output = Option<String>> + 'static,
{
    let mut editor = editor.clone();
    let mut dom_updates = dom_updates.clone();
    
    event.stop_propagation();
    event.prevent_default();

    match event.key() {
        Key::Character(_) => {
            spawn(async move {
                let (current_index_i, current_index_j, current_caret_pos) = editor.read().get_caret_pos().unwrap_or((index_i, index_j, 0));
                
                // Get current text from editor.raw_text with bounds checking
                let current_text = if current_index_i < editor.read().raw_text.len() && current_index_j < editor.read().raw_text[current_index_i].len() {
                    editor.read().raw_text[current_index_i][current_index_j].clone()
                } else {
                    String::new()
                };

                // Insert the character at the caret position
                if let Key::Character(ch) = &event.key() {
                    let char_str = ch.to_string();
                    let mut cur_text = current_text;
                    let caret_pos = current_caret_pos.min(cur_text.len());
                    cur_text.insert_str(caret_pos, &char_str);
                    let new_caret_pos = caret_pos + char_str.len();
                    
                    // Update editor state
                    editor.with_mut(|e| {
                        if current_index_i < e.raw_text.len() && current_index_j < e.raw_text[current_index_i].len() {
                            e.raw_text[current_index_i][current_index_j] = cur_text.clone();
                            e.move_caret(current_index_i, current_index_j, new_caret_pos);
                        }
                    });
                    
                    // Send updates
                    let tmp_cursor_pos = cur_text.len();
                    editor.write().update_text(current_index_i, current_index_j, cur_text.clone());
                    editor.write().move_caret(current_index_i, current_index_j , tmp_cursor_pos);

                    let id = format!("{},{},{}", current_index_i, current_index_j, new_caret_pos);
                    dom_updates.write().push_back(("update_text".to_string(), id, Some(cur_text)));
                }
            });
        }
        _ => {
            // Handle other keys if needed
        }
    }

    Ok(())
}

