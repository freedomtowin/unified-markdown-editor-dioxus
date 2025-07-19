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

    let focus_caret_position = use_coroutine(move |rx: UnboundedReceiver<(usize, usize)>| async move {
        coroutines::focus_caret_position_coroutine(rx, &mut editor).await;
    });

    
    // Combined DOM updates queue for text and DOM operations
    let mut dom_updates = use_signal(|| VecDeque::<(String, String, Option<String>)>::new());
    
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
                }
                "delete_row" => {
                    let js = format!(r#"return window.deleteRow('{}');"#, id);
                    let _ = document::eval(&js).await;
                }
                "create_row" => {
                    let insert_after = data.unwrap_or_default();
                    let js = if insert_after.is_empty() {
                        format!(r#"return window.createRow('{}');"#, id)
                    } else {
                        format!(r#"return window.createRow('{}', '{}');"#, id, insert_after)
                    };
                    let _ = document::eval(&js).await;
                }
                "delete_element" => {
                    let js = format!(r#"return window.deleteElement('{}');"#, id);
                    let _ = document::eval(&js).await;
                }
                "create_cell" => {
                    if let Some(cell_data) = data {
                        // Format: "row_id,text_content,cell_style"
                        let parts: Vec<&str> = cell_data.split(',').collect();
                        if parts.len() >= 3 {
                            let row_id = parts[0];
                            let text_content = parts[1];
                            let cell_style = parts[2];
                            let js = format!(r#"return window.createCell('{}', '{}', '{}', '{}');"#, 
                                id, row_id, text_content, cell_style);
                            let _ = document::eval(&js).await;
                        }
                    }
                }
                _ => {
                    println!("Unknown DOM operation: {}", operation);
                }
            }
        }
        _ => {
            tokio::time::sleep(Duration::from_micros(10)).await;
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
                    visual_editor.set(editor.read().raw_text.clone());
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

    async fn change_text(index_i: usize, index_j: usize, cursor_pos: usize, new_text: String) {
        
        let element_id = get_element_id(index_i, index_j);
        let text_b64 = general_purpose::STANDARD.encode(new_text.clone());

        let js = format!(
            r#"
            return window.clearElementText('{}', atob("{}"), {});
            "#,
            element_id,
            text_b64,
            cursor_pos
        );

        let _ = document::eval(&js).await;

        return;
    };



    // Function to focus an element by ID
    let focus_element = move |index_i: usize, index_j: usize, cursor_pos: usize| {

        let element_id = get_element_id(index_i, index_j);
        
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(5)).await;
            let js = format!(
                r#"
                return window.focusElementAndSetCaret('{}', {});
                "#,
                element_id,
                cursor_pos
            );
            let _ = document::eval(&js).await;

            tokio::time::sleep(Duration::from_millis(5)).await;
            focus_caret_position.send((index_i, index_j));
            
        });
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
    let update_syntax = move || {
        
        async move {
            
            while (*dom_updates.read()).len() > 0 {
                println!("waiting for updates {}", (*dom_updates.read()).len());
                tokio::time::sleep(Duration::from_micros(20)).await;
            }

            let input_text = editor.read().raw_text.clone();
        
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
                while editor.read().text_width.len() <= cur_index_i {
                    editor.write().text_width.push(vec![None]);
                }
                
                while editor.read().text_width[cur_index_i].len() <= cur_index_j {
                    editor.write().text_width[cur_index_i].push(None);
                }
                
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
                            
                            // Clear columns that exceed the new column count
                            if old_col_count > new_col_count {
                                for col_idx in new_col_count..old_col_count {
                                    delete_element(cur_index_i, col_idx);
                                    // editor_updates.write().push_back((cur_index_i, col_idx, 0, String::new()));
                                }

                                focus_element(cur_index_i, new_col_count - 1, editor.read().raw_text[cur_index_i][new_col_count - 1].len() - 1);
                            }
                            
                            editor.write().raw_text[cur_index_i] = syntax_text[cur_index_i].clone();
                            let col_num = syntax_text[cur_index_i].len();
                            if col_num > 1 {
                                for col_indx in 0..col_num.saturating_sub(1) {
                                    let prev_text = syntax_text[cur_index_i][col_indx].clone();
                                    // Send to dom_updates instead of editor_updates
                                    let id = format!("{},{},{}", cur_index_i, col_indx, prev_text.len());

                                    let tmp_cursor_pos = prev_text.len();
                                    editor.write().update_text(cur_index_i, col_indx, prev_text.clone());
                                    editor.write().move_caret(cur_index_i, col_indx , tmp_cursor_pos);

                                    dom_updates.write().push_back(("update_text".to_string(), id, Some(prev_text.clone())));
  
                                    measure_width.send((cur_index_i, col_indx, prev_text.clone()));
                            
                                    println!("current index text {} {} {:?} {}", cur_index_i, col_indx, prev_text, prev_text.len());
                                }
                            }
                        }
                        
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
                                    
                                    editor.write().raw_text[index_i][index_j] = cur_text.clone();
                                    
                                    let tmp_cursor_pos = cur_text.len();
                                    editor.write().update_text(index_i, index_j, cur_text.clone());
                                    editor.write().move_caret(index_i, index_j , tmp_cursor_pos);
                                    
                                    // Send to dom_updates instead of editor_updates
                                    let id = format!("{},{},{}", index_i, index_j, cur_text.len());
                                    dom_updates.write().push_back(("update_text".to_string(), id, Some(cur_text.to_string())));

                                    // focus_element(index_i, index_j, cur_text.len());
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

            visual_editor.set(editor.read().raw_text.clone());

            println!("input text: {:?}", input_text);
            println!("read text: {:?}", editor.read().raw_text);
            println!("raw text: {:?}", syntax_text);

            
        }
    };

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
                    spawn(async move {

                        tokio::time::sleep(Duration::from_micros(20)).await;
                        let _ = update_syntax().await;
                    });   
                    
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
