mod markdown;
mod state;
mod builder;
mod handler;

use dioxus::prelude::*;
use dioxus::events::Key;
use markdown::MarkdownRenderer;
use tokio;
use std::time::Duration;

use dioxus::desktop::{Config, WindowBuilder, LogicalSize};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, Level};

use state::State;
use builder::EditorBuilder;

static FUNCTIONS_JS: Asset = asset!("/assets/functions.js");
  
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

#[component]
fn App() -> Element {

    let raw_text = use_signal(|| "# First line\n\n# Second line\n\nTest\n".replace("\n\n","\r\n").to_string());
    let caret_pos = use_signal(|| None::<usize>);
    let undo_stack = use_signal(|| Vec::<String>::new());
    let selection_range = use_signal(|| None::<(usize, usize)>);
    let last_key_entered = use_signal(|| None::<Key>);

    let functions_js = use_memo(move || {
        read_file_or_fallback(
            FUNCTIONS_JS.resolve(), 
            FUNCTIONS_JS.bundled().absolute_source_path().into()
        ).unwrap()
    });



    // use_context_provider(|| Signal::new(EditorBuilder::new(Some(uci_tx))));
    let uci_tx = use_coroutine(|mut rx: UnboundedReceiver<String>| async move {
        
        while let  Ok(Some(msg)) = rx.try_next() {
            debug!("Editor reports: {msg}");
        }
    });

    // let mut move_builder = use_context::<Signal<EditorBuilder>>();
    let mut move_editor = use_signal(|| {

        let state = State::new(
            raw_text.clone(),
            caret_pos.clone(),
            undo_stack.clone(),
            selection_range.clone(),
            last_key_entered.clone()
        );
        EditorBuilder::new(Some(uci_tx), state)
    });


    // Coroutine that listens for new caret positions and updates the DOM selection.
    let caret_queue = use_coroutine(move |mut rx: UnboundedReceiver<usize>| async move {

        
        loop {
            // Try to receive a message
            match rx.try_next() {
                Ok(Some(pos)) => {
                    let js_code = functions_js();
                    let js_code_preview = format!("{} return copyPreviewToEditor()", &js_code);
    
                        
                    if let Ok(js_result) = document::eval(&js_code_preview).await {
                        println!("inner html: {:?}", js_result);
                    }
                    
                    tokio::time::sleep(Duration::from_millis(20)).await;

                    let last_key = (*move_editor.read()).last_key_entered.read().clone();

                    let mut last_key_was_enter = false;
                    if last_key == Some(Key::Enter) {
                        last_key_was_enter = true;
                    }
                    
                    // println!("caret que {:?}", pos);

                    
                    let js_code_cursor = format!("{} return insertCursorAtPosition({}, {})", &js_code, pos, last_key_was_enter);

                    // Introduce a small delay so the DOM can be updated first
                    // tokio::time::sleep(Duration::from_millis(10)).await;
                    
                    document::eval(&js_code_cursor).await.ok();
                }
                Ok(None) => {
                    // No messages available, wait for a short time before trying again
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    // Channel is closed, exit the loop
                    // println!("Channel closed");
                    // break;
                }
            }
        }
    });




    let transfer_input = use_coroutine(move |mut rx: UnboundedReceiver<String>| {
        async move {
            println!("Coroutine is running");
    
            loop {
                // Try to receive a message
                match rx.try_next() {
                    Ok(Some(message)) => {
                        let js_code = functions_js();
                        let js_code = format!("{} return copyPreviewToEditor()", &js_code);
    
                        
                        if let Ok(js_result) = document::eval(&js_code).await {
                            println!("inner html: {:?}", js_result);
                        //     let new_text = js_result.as_str()
                        //         .unwrap_or_default()
                        //         .replace('\u{200b}', "");  // Remove zero-width spaces
                        //     println!("updating view: {:?}", new_text);
                        //     let raw_text_read = move_editor.read().raw_text.read().clone();
                        //     if !new_text.is_empty() && new_text != raw_text_read {
                        //         move_editor.write().raw_text.set(new_text.to_string());
                        //     }
                        }
                    }
                    Ok(None) => {
                        // No messages available, wait for a short time before trying again
                        tokio::time::sleep(Duration::from_millis(20)).await;
                    }
                    Err(_) => {
                        tokio::time::sleep(Duration::from_millis(20)).await;
                        // Channel is closed, exit the loop
                        // println!("Channel closed");
                        // break;
                    }
                }
            }
    
            println!("Coroutine finished");
        }
    });

    

    let caret_click = use_coroutine(move |mut rx: UnboundedReceiver<String>| {
        async move {
            println!("Coroutine is running");
    
            loop {
                // Try to receive a message
                match rx.try_next() {
                    Ok(Some(message)) => {

                        let js_code = functions_js();
                        let js_code = format!("{} return getCaretClickPosition()", &js_code);
                        // println!("{}", js_code);
                        if let Err(e) = document::eval(&js_code).await {
                            println!("JS eval error: {:?}", e); // Add this line
                        } else if let Ok(result) = document::eval(&js_code).await {
                            if let Ok(pos) = result.to_string().parse::<usize>() {
                                println!("click {:?}", pos);
                                move_editor.write().set_caret(pos);
                            }
                        }
                    }
                    Ok(None) => {
                        // No messages available, wait for a short time before trying again
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                    Err(_) => {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        // Channel is closed, exit the loop
                        // println!("Channel closed");
                        // break;
                    }
                }
            }
    
            println!("Coroutine finished");
        }
    });

    //   let _sync_task = use_coroutine(move |_rx: UnboundedReceiver<()>| async move {
    //     loop {
    //         // let js_code = functions_js();
    //         // let js_code = format!("{} return copyPreviewToEditor()", &js_code);

            
    //         // if let Ok(js_result) = document::eval(&js_code).await {
    //         //     println!("inner html: {:?}", js_result);
    //         // }

    //         let car = *(*move_editor.read()).caret_pos.read();
    //         if let Some(pos) = car {
    //             caret_queue.send(pos);
    //         }
    //         tokio::time::sleep(Duration::from_millis(100)).await;
    //     }
    // });



    let renderer = use_memo(move || {
        let mut renderer = MarkdownRenderer::new(move_editor.read().get_raw_text());
        renderer.render_to_elements();
        renderer
    });

    rsx! {
        div {
            style: "display: flex; gap: 20px; padding: 10px;",

            // Editor Pane: A contenteditable div showing the raw markdown text.
            div {
                style: "flex: 1; border: 1px solid #ccc; padding: 1px; height: 60%",
                h3 { "Editor (Content Editable)" }
                // The contenteditable div uses an id ("editor") for JS interop.
                div {
                    id: "editor",
                    contenteditable: "true",
                    style: "overflow-y: auto;  border: 1px solid #aaa; padding: 8px;",
                    onkeydown: move |evt| {

                        
                        let mut handler = move_editor.write();


                        
                        handler.handle_keydown(evt);
                    
                        // let car = *handler.caret_pos.read();
                        // if let Some(pos) = car {
                        //     caret_queue.send(pos);
                        // }      
                        // transfer_input.send("update".to_string());

                        let car = *handler.caret_pos.read();
                        if let Some(pos) = car {
                            caret_queue.send(pos);
                        }
                        
                        
                    },
                    oninput: move |_| {
                        
                        // .raw_text.read().clone()
                        // move_editor.write().handle_input();
                    },
                    onmouseup: move |_| {
                        println!("Sending click event"); // Debug print
                        caret_click.send("click".to_string());

                    },
                    onclick: move |_| {
                        caret_click.send("click".to_string());

                    },
                    // Here we simply display the raw text.
                    // In a more advanced version you might run a syntax highlighter
                    // to wrap tokens in spans for color/styling.

                    { renderer().nodes.clone().into_iter() }
          
                }
            },

            // Preview Pane: A read-only live preview rendered using MarkdownRenderer.
            div {
                id: "preview",
                
                style: "display: none; flex: 1; border: 1px solid #ccc; padding: 8px; white-space: pre-wrap;",
                { renderer().nodes.clone().into_iter() }
                // ""
            }
        },
        // Debug info: show raw state and caret position.
        div {
            style: "margin-top: 1em; font-family: monospace;",
            "Raw Text: ",
            pre { {format!("{}", move_editor.read().raw_text.clone())} },
            "Caret Position: ",
            {
            if let Some(pos) = move_editor.read().get_caret_position() {
                rsx!{ "{pos}" }
            } else {
                rsx!{ "None" }
            }}
            br {},
            "Selection Range: ",
            {
                if let Some((start, end)) = move_editor.read().get_selection_range() {
                    rsx! { "Start: {start}, End: {end}" }
                } else {
                    rsx! { "None" }
                }
            }            
        }
    }
}


fn read_file_or_fallback(primary_path: PathBuf, fallback_path: PathBuf) -> Option<String> {
    if primary_path.exists() {
        fs::read_to_string(primary_path).ok()
    } else {
        fs::read_to_string(fallback_path).ok()
    }
}