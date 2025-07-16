// mod markdown;
mod state;
mod builder;
mod handler;
mod coroutines;
mod syntax;

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
            input_text,
            None,
        ),
    ));

    let mut is_mouse_down = use_signal(|| false);
    let mut is_mouse_dragging = use_signal(|| false);
    let mut last_cursor_position = use_signal(|| 0.0);
    

    let measure_width = use_coroutine(move |rx| async move {
        coroutines::measure_width_coroutine(rx, &mut editor).await;
    });

    let focus_caret_position = use_coroutine(move |rx: UnboundedReceiver<(usize, usize)>| async move {
        coroutines::focus_caret_position_coroutine(rx, &mut editor).await;
    });

    let update_editor_text = use_coroutine(move |mut rx: UnboundedReceiver<(usize, usize, usize, String)>| async move {
        coroutines::update_editor_text_coroutine(rx, &mut editor).await;
    });

    let mut editor_updates = use_signal(|| VecDeque::<(usize, usize, usize, String)>::new());

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
            tokio::time::sleep(Duration::from_micros(100)).await;
            if (*editor_updates.read()).len() > 0 {
                
                if let Some((index_i, index_j, pos, text)) = (*editor_updates.write()).pop_front() {        
   

                    editor.with_mut(|e| e.move_caret(index_i, index_j, pos));
                    measure_width.send((index_i, index_j, text.clone()));

                    editor.with_mut(|e| e.update_text(index_i, index_j, text.clone()));
                    update_editor_text.send((index_i, index_j, pos, text.clone()));
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

    // let change_text = move |index_i: usize, index_j: usize, cursor_pos: usize, new_text: String| {
        
    //     async move {
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
    //     }
    // };



    // Function to focus an element by ID
    let focus_element = move |index_i: usize, index_j: usize, cursor_pos: usize| {

        let element_id = get_element_id(index_i, index_j);
        
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(25)).await;
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

            let _ = handler::handle_enter_key(event, index_i, index_j, &editor, measure_width, editor_updates);
        //     spawn(async move {
        //         // tokio::time::sleep(Duration::from_millis(100)).await;
        //         editor.with_mut(|e| {
        //             let (caret_i, caret_j, caret_pos) = e.get_caret_pos().unwrap_or((index_i, index_j, 0));
        //             if caret_i == index_i && caret_j == index_j {
        //                 let current_text = e.raw_text[index_i][index_j].clone();

        //                 let (before, after) = current_text.split_at(caret_pos);

        //                 e.update_text(index_i, index_j, before.to_string());
        //                 e.raw_text.insert(index_i+1, vec![after.to_string()]);
                        
        //                 measure_width.send((index_i, index_j, before.to_string()));
        //                 measure_width.send((index_i, index_j + 1, after.to_string()));

        //                 // update_editor_text.send((index_i, index_j, before.len(), before.to_string()));
        //                 // update_editor_text.send((index_i + 1, 0, after.len(), after.to_string()));
        //                 editor_updates.write().push_back((index_i, index_j, before.len(), before.to_string()));
        //                 editor_updates.write().push_back((index_i + 1, 0, after.len(), after.to_string()));
                        
        //             }
        //         });
        // });
        } else if event.key() == Key::ArrowLeft {

            println!("[left arrow] current text {}, len {}", cur_text, cur_text.len());
            // println!("[left arrow] current indx {}", indx_j);
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

                // let editor_text = get_editor_text(current_index_i, current_index_j).await;
                // let current_text = editor.read().raw_text[current_index_i][current_index_j].to_string();
                                                    
                

                    // println!("[handle keydown] {} {}", e_pos, current_caret_pos);

                    // if (current_index_i == index_i) && (current_index_j == index_j) && (e_pos == current_caret_pos) {
                    //     let (index_i, index_j, current_caret_pos) = editor.read().get_caret_pos().unwrap_or((index_i, index_j, 0));



                    // }
                }

                

                
            });
        } else if event.key() == Key::ArrowRight {

    
            spawn(async move {
                // tokio::time::sleep(Duration::from_millis(100)).await;

                
                let (current_index_i, current_index_j, current_caret_pos) = editor.read().get_caret_pos().unwrap_or((index_i, index_j, 0));

                let editor_pos = get_editor_caret_position(current_index_i, current_index_j).await;

                if (current_index_i == index_i) && (current_index_j == index_j) {
                    
                    if let Some(e_pos) = editor_pos {

                    editor.with_mut(|e| e.move_caret(index_i, index_j, e_pos));
                        println!("[handle keydown] {} {}", e_pos, current_caret_pos);
                        // if (current_index_i == index_i) && (current_index_j == index_j) && (e_pos == current_caret_pos) {
                            
                            let current_text_len = editor.read().get_raw_text_current().len().saturating_sub(1);
                            let current_col_size = editor.read().raw_text[index_i].len().saturating_sub(1);
                            let current_row_size = editor.read().raw_text.len().saturating_sub(1);
            
                            editor.with_mut(|e| {
                                
                                let (new_index_i, new_index_j, new_pos) = if (current_caret_pos > current_text_len) && (index_j < current_col_size) {
                                    // If the is a previous cell in the same row 
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
            
                                // e.move_caret(new_index_i, new_index_j, new_pos);
                                let element_id = get_element_id(new_index_i, new_index_j);
                                if (new_index_j != index_j) || (new_index_i != index_i){
                                    focus_element(new_index_i, new_index_j, new_pos);
                                } else {
                                    focus_caret_position.send((new_index_i, new_index_j));
                                }
            
                                info!("prev index: {:?}, new_pos {}", format!("textarea-{}-{}", new_index_i, new_index_j), new_pos);
            
                        });
                    // }
                
                }
                    // tokio::time::sleep(Duration::from_millis(100)).await;

                    // focus_element(element_id, new_pos);
                    
                    // caret_click.send((new_index_i, new_index_j, element_id));
                }

            });
        } else {

            spawn(async move {


            let editor_pos = get_editor_caret_position(index_i, index_j).await;
            // if let Some(e_pos) = editor_pos {
            //     editor.with_mut(|e| e.move_caret(index_i, index_j, e_pos));
            // }
            let editor_text = get_editor_text(index_i, index_j).await;

            let (current_index_i, current_index_j, current_caret_pos) = editor.read().get_caret_pos().unwrap_or((index_i, index_j, 666));
        
            let current_text = editor.read().raw_text[current_index_i][current_index_j].to_string();

            // update_editor_text.send((index_i, index_j, current_text.clone(), current_caret_pos));


            if let Some(e_pos) = editor_pos {
                // println!("[handle keydown] {} {}", e_pos, current_caret_pos);
                if (e_pos as i32 - current_caret_pos as i32).abs() <= 5 {
                    // tokio::time::sleep(Duration::from_millis(10)).await;
                    // editor.with_mut(|e| e.move_caret(index_i, index_j, e_pos));
                    // update_editor_text.send((index_i, index_j, current_text.clone(), current_caret_pos));
                    

                    if let Some(e_text) = editor_text {
                        // update_editor_text.send((index_i, index_j, current_text.clone(), current_caret_pos));
                        // caret_click.send((index_i, index_j, e_pos, e_text))
                        update_editor_text.send((index_i, index_j, e_pos, e_text.clone()));
                        editor_updates.write().push_back((index_i, index_j, e_pos, e_text));
                    }

                }
                

            }
            //         let (_, _, current_caret_pos) = editor.read().get_caret_pos().unwrap_or((index_i, index_j, 667));
            //         tokio::time::sleep(Duration::from_millis(50)).await;
            //         println!("[handle keydown] {} {}", e_pos, current_caret_pos);
            //  }
            // }
            
            // let e = editor.read();
            



            });
            // println!("{:?}", editor.read().raw_text);
        }
    };
    
    // let renderer = use_memo(move || {
    //     editor.read().raw_text.clone()
    // });
    let update_syntax = move || {
        
        async move {
            tokio::time::sleep(Duration::from_millis(1)).await;
            let input_text = editor.read().raw_text.clone();
        
            // let editor_pos = get_editor_caret_position(index_i, index_j).await;

            let maybe_current_caret = editor.read().get_caret_pos();
            let row_level_caret_pos = editor.read().get_row_level_caret_pos(maybe_current_caret);
    
            // println!("{:?}", input_text);
            let text_processor = TextProcessor::new();
            let join_text = text_processor.markdown_to_string(input_text.clone());
            let syntax_text = text_processor.process_markdown(join_text.clone());
            let raw_text = text_processor.extract_strings(syntax_text.clone());
            
            if input_text ==  raw_text {
                return;
            }
            // assert_eq!(input_text, raw_text);
            // editor.write().raw_text = raw_text.clone();
    
            
            if let Some((cur_index_i, cur_index_j, cur_caret_pos)) = maybe_current_caret {

                println!("[update syntax] index i {} index j {} cursor {}", cur_index_i, cur_index_j, cur_caret_pos);
                
                if cur_index_i >= editor.read().text_width.len() {
                    editor.write().text_width.push(vec![None]);
                }           

                while cur_index_j >= editor.read().text_width[cur_index_i].len() - 1 {
                    // editor.write().text_width
                    editor.write().text_width[cur_index_i].push(None);
                }
                
                // This potentially needs to be robust
                let mut cur_index_j = cur_index_j;
                if cur_index_j > raw_text[cur_index_i].len() - 1 {
                    cur_index_j = raw_text[cur_index_i].len() - 1;
                }

                println!("row {:?}", raw_text[cur_index_i]);
                println!("{:?}", cur_index_j);
                
                let cur_text = raw_text[cur_index_i][cur_index_j].clone();

                editor.write().raw_text[cur_index_i] = raw_text[cur_index_i].clone();
                
                
                // let row_size = editor.write().raw_text[cur_index_i].len();
                // for (copy_index_j, col) in raw_text[cur_index_i].iter().enumerate() {

                //     editor_updates.write().push_back((cur_index_i, copy_index_j, col.len(), col.clone()));

                //     if copy_index_j >= row_size {
                //         editor.write().raw_text[cur_index_i].push(col.clone());
                //     }
                //     else {
                //     editor.write().raw_text[cur_index_i][index_j] = col.clone();
                //    }
                // }
                
                measure_width.send((cur_index_i, cur_index_j, cur_text.to_string()));                

                println!("current index text {} {} {:?}", cur_index_i, cur_index_j, cur_text);
                
                editor_updates.write().push_back((cur_index_i, cur_index_j, cur_caret_pos, cur_text));

                if let Some(global_row_caret_pos) = row_level_caret_pos {
                    let maybe_new_caret_pos = editor.read().get_caret_from_row_level_pos(global_row_caret_pos, cur_index_i, raw_text[cur_index_i].clone());
    
                    
                    println!("new caret: {:?}", maybe_new_caret_pos);
                    if let Some((index_i, index_j, char_pos)) = maybe_new_caret_pos {
                        // editor.write().move_caret(index_i, index_j, char_pos);
    
                        let cur_text = raw_text[index_i][index_j].clone();
    
                        editor_updates.write().push_back((index_i, index_j, char_pos, cur_text));
                    }
                }
            }
    
            println!("input text: {:?}", input_text);
            // println!("join text: {:?}", join_text);
            println!("raw text: {:?}", raw_text);
        }
    };

    let renderer = use_memo(move || {


        let input_text = editor.read().raw_text.clone();
        
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
                    if (*editor_updates.read()).len() < 20 { 
                        handle_keydown_input(event, row, col);
                    }
                    spawn(async move {

                        while (*editor_updates.read()).len() > 0 {
                            println!("waiting for updates {}", (*editor_updates.read()).len());
                            tokio::time::sleep(Duration::from_millis(1)).await;
                        }
                        tokio::time::sleep(Duration::from_millis(1)).await;
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
