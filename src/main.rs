mod markdown;

use dioxus::prelude::*;
use dioxus::events::{Key, KeyboardEvent};
use markdown::MarkdownRenderer;
use tokio;
use std::time::Duration;

use arboard::Clipboard;
use serde_json::Value;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    // A single source of truth for the raw Markdown text.
    let raw_text = use_signal(|| "# First line\n\n# Second line\nTest\n".to_string());
    // State to track the caret (cursor) position as a global offset.
    let mut caret_pos = use_signal(|| None::<usize>);
    
    let undo_stack = use_signal(|| Vec::<String>::new());

    // State to track a selection range (anchor, active). When None, no highlighting.
    let selection_range = use_signal(|| None::<(usize, usize)>);

    let editor_initialized = use_signal(|| false);
    let prev_raw_text = use_signal(|| String::new());

    let sync_editor_content = {
        move || {
            spawn(async move {
                let js_code = r#"
                    // Copy preview HTML to editor
                    const preview = document.getElementById('preview');
                    const editor = document.getElementById('editor');
                    editor.innerHTML = preview.innerHTML;
                "#;
                document::eval(js_code).await.ok();
            });
        }
    };




    // Create a preview by re-parsing the raw markdown via your MarkdownRenderer.
    let preview_nodes = use_memo( move || {
        let mut renderer = MarkdownRenderer::new(raw_text.read().clone());
        renderer.render_to_elements();
        renderer.nodes
    });

   
  

    // Add this near your state declarations
    let caret_queue = use_coroutine(|mut rx: UnboundedReceiver<usize>| async move {
        while let Ok(Some(pos)) = rx.try_next() {
            let js_code = format!(
                r#"
                (function() {{
                    const el = document.getElementById("editor");
                    if (!el) return;
                    
                    el.focus();
                    const sel = window.getSelection();
                    sel.removeAllRanges();
                    
                    const range = document.createRange();
                    const textNode = el.firstChild || document.createTextNode('');
                    range.setStart(textNode, {pos});
                    range.collapse(true);
                    sel.addRange(range);
                }})();
                "#,
                pos = pos
            );
            tokio::time::sleep(Duration::from_millis(20)).await; // Increased delay
            document::eval(&js_code).await.ok();
        }
    });

    // Modified set_caret to use the queue
    let set_caret = {
        let caret_queue = caret_queue.clone();
        move || {
            if let Some(pos) = *caret_pos.read() {
                caret_queue.send(pos);
            }
        }
    };

    // Updated update_caret function with a delay.
    let update_caret = {
        move || {
            spawn(
                async move {
                    let js_code = r#"
                        (function() {
                            let el = document.getElementById("editor");
                            if (!el) return -1;
                            let sel = window.getSelection();
                            if (!sel || sel.rangeCount === 0) return -1;
                            let range = sel.getRangeAt(0);
                            let preCaretRange = range.cloneRange();
                            preCaretRange.selectNodeContents(el);
                            preCaretRange.setEnd(range.endContainer, range.endOffset);
                            return preCaretRange.toString().length;
                        })();
                    "#;
                    if let Ok(result) = document::eval(js_code).await {
                        if let Ok(pos) = result.to_string().parse::<isize>() {
                            let pos = if pos < 0 { 0 } else { pos as usize };
                            caret_pos.with_mut(|cp| *cp = Some(pos));
                            caret_queue.send(pos);
                            println!("update caret {:?}", caret_pos.read());
                        }
                    }
                }
            );
        }
    };

    // Modify the sync task to use the LATEST caret position
    let _sync_task = use_coroutine(move |_rx: UnboundedReceiver<()>| async move {
        loop {
            // Get the position RIGHT BEFORE syncing
            let current_pos = *caret_pos.read();
            
            // First update the editor content
            let js_sync = r#"
                const preview = document.getElementById('preview');
                const editor = document.getElementById('editor');
                editor.innerHTML = preview.innerHTML;
            "#;
            document::eval(js_sync).await.ok();

            // Then set caret position if needed
            if let Some(pos) = current_pos {
                let js_caret = format!(
                    r#"
                    const range = document.createRange();
                    const sel = window.getSelection();
                    range.setStart(editor.firstChild, {pos});
                    range.collapse(true);
                    sel.removeAllRanges();
                    sel.addRange(range);
                    "#
                );
                document::eval(&js_caret).await.ok();
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    let update_caret_click = {
        let caret_queue = caret_queue.clone();
        move |evt: MouseEvent| {
            let coords = evt.client_coordinates();
            
            spawn(async move {
                let js_code = format!(
                    r#"
                    (function() {{
                        try {{
                            const editor = document.getElementById('editor');
                            if (!editor) return {{ error: "No editor element" }};
                            
                            const rect = editor.getBoundingClientRect();
                            const clickX = {x} + window.scrollX;
                            const clickY = {y} + window.scrollY;
                            
                            let range;
                            if (document.caretRangeFromPoint) {{
                                range = document.caretRangeFromPoint(clickX, clickY);
                            }} else {{
                                const pos = document.caretPositionFromPoint(clickX, clickY);
                                if (!pos) return {{ error: "No caret position" }};
                                range = document.createRange();
                                range.setStart(pos.offsetNode, pos.offset);
                            }}
                            
                            if (!range || !editor.contains(range.startContainer)) {{
                                return {{ error: "Invalid range" }};
                            }}
                            
                            let offset = 0;
                            const walker = document.createTreeWalker(
                                editor,
                                NodeFilter.SHOW_TEXT,
                                null,
                                false
                            );
                            
                            let node = walker.nextNode();
                            while (node && node !== range.startContainer) {{
                                offset += node.textContent.length;
                                node = walker.nextNode();
                            }}
                            
                            offset += range.startOffset;
                            return {{ offset: offset }};
                            
                        }} catch (e) {{
                            return {{ error: e.toString() }};
                        }}
                    }})()
                    "#,
                    x = coords.x as f64,
                    y = coords.y as f64
                );

                match document::eval(&js_code).await {
                    Ok(result) => {
                        let json_str = result.to_string();
                        match serde_json::from_str::<Value>(&json_str) {
                            Ok(json) => {
                                if let Some(offset) = json["offset"].as_u64() {
                                    caret_pos.with_mut(|cp| *cp = Some(offset as usize));
                                    caret_queue.send(offset as usize);
                                }
                            }
                            Err(_) => println!("Invalid JSON response: {}", json_str),
                        }
                    }
                    Err(e) => println!("JS execution error: {:?}", e),
                }
            });
        }
    };





    // Key handler: intercept key events and update our raw_text and caret.
    let handle_keydown = {
        let mut raw_text = raw_text.clone();
        let mut caret_pos = caret_pos.clone();
        let mut selection_range = selection_range.clone();
        let update_caret = update_caret.clone();
        let set_caret = set_caret.clone();
        let mut undo_stack = undo_stack.clone();
        move |evt: KeyboardEvent| {
            let text = raw_text.read().clone();
            let pos = caret_pos.read().unwrap_or(0);

            // update_caret();

            // If CTRL is pressed, handle CTRL shortcuts first.
            // If CTRL is pressed, handle CTRL shortcuts first.
            if evt.data().modifiers().ctrl() {
                let key_lower = evt.data().key().to_string();
                match key_lower.as_str() {
                    "z" => {
                        evt.prevent_default();
                        if let Some(previous) = undo_stack.write().pop() {
                            raw_text.set(previous);
                        }
                    }
                    "c" => {
                        evt.prevent_default();
                        if let Some((sel_start, sel_end)) = *selection_range.read() {
                            let s = sel_start.min(sel_end);
                            let e = sel_start.max(sel_end);
                            let selected_text = &raw_text.read()[s..e];

                            let mut clipboard = Clipboard::new().expect("Failed to open clipboard");
                            clipboard.set_text(selected_text.to_string()).expect("Failed to copy to clipboard");
                        }
                    }
                    "v" => {
                        evt.prevent_default();
                        let mut clipboard = Clipboard::new().expect("Failed to open clipboard");
                        if let Ok(paste_text) = clipboard.get_text() {
                            let current = raw_text.read().clone();
                            let pos = caret_pos.read().unwrap_or(0);
                            undo_stack.write().push(current.clone());
                            let (left, right) = current.split_at(pos);
                            let new_text = format!("{}{}{}", left, paste_text, right);
                            raw_text.set(new_text);

                            caret_pos.with_mut(|cp| *cp = Some(pos + paste_text.len()));
                            caret_queue.send(pos + paste_text.len());
                        }
                    }
                    "x" => {
                        evt.prevent_default();
                        let sel_range = *selection_range.read();
                        if let Some((sel_start, sel_end)) = sel_range {
                            let s = sel_start.min(sel_end);
                            let e = sel_start.max(sel_end);
                            let selected_text = &raw_text.read().clone()[s..e];

                            let mut clipboard = Clipboard::new().expect("Failed to open clipboard");
                            clipboard.set_text(selected_text.to_string()).expect("Failed to copy to clipboard");

                            // Delete selected text after copying
                            undo_stack.write().push(raw_text.read().clone());
                            let current = raw_text.read().clone();
                            let new_text = format!("{}{}", &current[0..s], &current[e..]);
                            raw_text.set(new_text);
                            caret_pos.with_mut(|cp| *cp = Some(s));
                            caret_queue.send(s);
                            selection_range.set(None);
                        }
                    }
                    _ => {}
                }
                set_caret();
                // caret_queue.send();
                return;
            }

            // If Shift is pressed, handle selection highlighting.
            if evt.data().modifiers().shift() && matches!(evt.data().key(), Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End) {
                match evt.data().key() {
                    Key::ArrowLeft => {
                        evt.prevent_default();
                        if pos > 0 {
                            let new_pos = pos - 1;
                            caret_pos.with_mut(|cp| *cp = Some(new_pos));
                            
                            caret_queue.send(new_pos);
                            selection_range.with_mut(|sel_range| {
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
                        if pos < text.len() {
                            let new_pos = pos + 1;
                            caret_pos.with_mut(|cp| *cp = Some(new_pos));
                            
                            caret_queue.send(new_pos);
                            selection_range.with_mut(|sel_range| {
                                if let Some((anchor, _)) = *sel_range {
                                    *sel_range = Some((anchor, new_pos));
                                } else {
                                    *sel_range = Some((pos, new_pos));
                                }
                            });
                        }
                    }
                    Key::ArrowUp => {
                        evt.prevent_default();
                        let lines: Vec<&str> = text.split('\n').collect();
                        let mut cumulative = 0;
                        let mut current_line = 0;
                        for (i, line) in lines.iter().enumerate() {
                            if pos >= cumulative && pos <= cumulative + line.len() {
                                current_line = i;
                                break;
                            }
                            cumulative += line.len() + 1;
                        }
                        if current_line > 0 {
                            let col = pos - cumulative;
                            let prev_line = lines[current_line - 1];
                            let new_col = col.min(prev_line.len());
                            let mut new_pos = 0;
                            for i in 0..(current_line - 1) {
                                new_pos += lines[i].len() + 1;
                            }
                            new_pos += new_col;
                            caret_pos.with_mut(|cp| *cp = Some(new_pos));
                            
                            caret_queue.send(new_pos);
                            selection_range.with_mut(|sel_range| {
                                if let Some((anchor, _)) = *sel_range {
                                    *sel_range = Some((anchor, new_pos));
                                } else {
                                    *sel_range = Some((pos, new_pos));
                                }
                            });
                        }
                    }
                    Key::ArrowDown => {
                        evt.prevent_default();
                        let lines: Vec<&str> = text.split('\n').collect();
                        let mut cumulative = 0;
                        let mut current_line = 0;
                        for (i, line) in lines.iter().enumerate() {
                            if pos >= cumulative && pos <= cumulative + line.len() {
                                current_line = i;
                                break;
                            }
                            cumulative += line.len() + 1;
                        }
                        if current_line < lines.len() - 1 {
                            let col = pos - cumulative;
                            let next_line = lines[current_line + 1];
                            let new_col = col.min(next_line.len());
                            let mut new_pos = cumulative + lines[current_line].len() + 1;
                            new_pos += new_col;
                            caret_pos.with_mut(|cp| *cp = Some(new_pos));
                            
                            caret_queue.send(new_pos);
                            selection_range.with_mut(|sel_range| {
                                if let Some((anchor, _)) = *sel_range {
                                    *sel_range = Some((anchor, new_pos));
                                } else {
                                    *sel_range = Some((pos, new_pos));
                                }
                            });
                        }
                    }
                    Key::Home => {
                        evt.prevent_default();
                        let lines: Vec<&str> = text.split('\n').collect();
                        let mut cumulative = 0;
                        for line in &lines {
                            if pos >= cumulative && pos <= cumulative + line.len() {
                                caret_pos.with_mut(|cp| *cp = Some(cumulative));
                                
                                caret_queue.send(cumulative);
                                selection_range.with_mut(|sel_range| {
                                    if let Some((anchor, _)) = *sel_range {
                                        *sel_range = Some((anchor, cumulative));
                                    } else {
                                        *sel_range = Some((pos, cumulative));
                                    }
                                });
                                break;
                            }
                            cumulative += line.len() + 1;
                        }
                    }
                    Key::End => {
                        evt.prevent_default();
                        let lines: Vec<&str> = text.split('\n').collect();
                        let mut cumulative = 0;
                        for line in &lines {
                            if pos >= cumulative && pos <= cumulative + line.len() {
                                let new_pos = cumulative + line.len();
                                caret_pos.with_mut(|cp| *cp = Some(new_pos));
                                
                                caret_queue.send(new_pos);
                                selection_range.with_mut(|sel_range| {
                                    if let Some((anchor, _)) = *sel_range {
                                        *sel_range = Some((anchor, new_pos));
                                    } else {
                                        *sel_range = Some((pos, new_pos));
                                    }
                                });
                                break;
                            }
                            cumulative += line.len() + 1;
                        }
                    }
                    _ => {}
                }
                set_caret();
                return;
            }


            // Normal key handling (non-CTRL, non-SHIFT)
            match evt.data.key() {
                // Insert newline at the caret when Enter is pressed.
                Key::Enter => {
                    evt.prevent_default();
                    let (left, right) = text.split_at(pos);
                    undo_stack.write().push(text.clone());
                    let new_text = format!("{}\n{}", left, right);
                    raw_text.set(new_text);

                    let new_pos = pos + 1;
                    caret_pos.with_mut(|cp| {*cp = Some(new_pos);});
                    
                    caret_queue.send(new_pos);
                }
                // Remove character before caret for Backspace.
                // Remove character before caret for Backspace.
                Key::Backspace => {
                    evt.prevent_default();
                    let sel_range = *selection_range.read();
                    if let Some((sel_start, sel_end)) = sel_range {
                        let s = sel_start.min(sel_end);
                        let e = sel_start.max(sel_end);
                        undo_stack.write().push(text.clone());
                        let new_text = format!("{}{}", &text[0..s], &text[e..]);
                        raw_text.set(new_text);
                        caret_pos.with_mut(|cp| *cp = Some(s));
                        caret_queue.send(s);
                        selection_range.set(None);
                    } else if pos > 0 {
                        let mut chars: Vec<char> = text.chars().collect();
                        undo_stack.write().push(text.clone());
                        chars.remove(pos - 1);
                        let new_text: String = chars.into_iter().collect();
                        raw_text.set(new_text);
                        caret_pos.with_mut(|cp| {
                            *cp = Some(pos - 1);
                        });
                        caret_queue.send(pos - 1);
                    }
                },
                // Delete key: remove character after caret.
                Key::Delete => {
                    evt.prevent_default();
                    let sel_range = *selection_range.read();
                    if let Some((sel_start, sel_end)) = sel_range {
                        let s = sel_start.min(sel_end);
                        let e = sel_start.max(sel_end);
                        undo_stack.write().push(text.clone());
                        let new_text = format!("{}{}", &text[0..s], &text[e..]);
                        raw_text.set(new_text);
                        caret_pos.with_mut(|cp| *cp = Some(s));
                        caret_queue.send(s);

                        selection_range.set(None);
                    } else if pos < text.len() {
                        let mut chars: Vec<char> = text.chars().collect();
                        undo_stack.write().push(text.clone());
                        chars.remove(pos);
                        let new_text: String = chars.into_iter().collect();
                        raw_text.set(new_text);
                        caret_pos.with_mut(|cp| *cp = Some(pos));
                        
                        caret_queue.send(pos);
                    }
                },
                // Move caret left.
                Key::ArrowLeft => {
                    evt.prevent_default();
                    if pos > 0 {
                        caret_pos.with_mut(|cp| *cp = Some(pos - 1));
                        caret_queue.send(pos - 1);
                    }
                }
                // Move caret right.
                Key::ArrowRight => {
                    evt.prevent_default();
                    if pos < text.len() {
                        caret_pos.with_mut(|cp| *cp = Some(pos + 1));
                        caret_queue.send(pos + 1);
                    }
                }
                // Move caret up.
                Key::ArrowUp => {
                    evt.prevent_default();
                    let lines: Vec<&str> = text.split('\n').collect();
                    let mut cumulative = 0;
                    let mut current_line = 0;
                    for (i, line) in lines.iter().enumerate() {
                        if pos >= cumulative && pos <= cumulative + line.len() {
                            current_line = i;
                            break;
                        }
                        cumulative += line.len() + 1;
                    }
                    if current_line > 0 {
                        let col = pos - cumulative;
                        let prev_line = lines[current_line - 1];
                        let new_col = col.min(prev_line.len());
                        let mut new_pos = 0;
                        for i in 0..(current_line - 1) {
                            new_pos += lines[i].len() + 1;
                        }
                        new_pos += new_col;
                        caret_pos.with_mut(|cp| *cp = Some(new_pos));
                        
                        caret_queue.send(new_pos);
                    }
                }
                // Move caret down.
                Key::ArrowDown => {
                    evt.prevent_default();
                    let lines: Vec<&str> = text.split('\n').collect();
                    let mut cumulative = 0;
                    let mut current_line = 0;
                    for (i, line) in lines.iter().enumerate() {
                        if pos >= cumulative && pos <= cumulative + line.len() {
                            current_line = i;
                            break;
                        }
                        cumulative += line.len() + 1;
                    }
                    if current_line < lines.len() - 1 {
                        let col = pos - cumulative;
                        let next_line = lines[current_line + 1];
                        let new_col = col.min(next_line.len());
                        let mut new_pos = cumulative + lines[current_line].len() + 1;
                        new_pos += new_col;
                        caret_pos.with_mut(|cp| *cp = Some(new_pos));
                        
                        caret_queue.send(new_pos);
                    }
                }
                // Home: move caret to beginning of current line.
                Key::Home => {
                    evt.prevent_default();
                    let lines: Vec<&str> = text.split('\n').collect();
                    let mut cumulative = 0;
                    for line in &lines {
                        if pos >= cumulative && pos <= cumulative + line.len() {
                            caret_pos.with_mut(|cp| *cp = Some(cumulative));
                            
                            caret_queue.send(cumulative);
                            break;
                        }
                        cumulative += line.len() + 1;
                    }
                }
                // End: move caret to end of current line.
                Key::End => {
                    evt.prevent_default();
                    let lines: Vec<&str> = text.split('\n').collect();
                    let mut cumulative = 0;
                    for line in &lines {
                        if pos >= cumulative && pos <= cumulative + line.len() {
                            caret_pos.with_mut(|cp| *cp = Some(cumulative + line.len()));
                            
                            caret_queue.send(cumulative + line.len());
                            break;
                        }
                        cumulative += line.len() + 1;
                    }
                }
                // For printable characters, insert them at the current position.
                _ => {
                    if let Key::Character(ch) = &evt.data.key() {
                        undo_stack.with_mut(|stack| stack.push(text.clone()));
                        let (left, right) = text.split_at(pos);
                        let new_text = format!("{}{}{}", left, ch, right);
                        raw_text.set(new_text);
                        
                        let new_pos = pos + ch.len();
                        caret_pos.with_mut(|cp| *cp = Some(new_pos));
                        caret_queue.send(new_pos);
                    }
                }
            }
            selection_range.with_mut(|sel_range| {
                    
                *sel_range = None;
                
            });            
        
            return;



        }
    };

    let handle_input = {
        let mut raw_text = raw_text.clone();
        let mut prev_raw_text = prev_raw_text.clone();
        move |_| {
            spawn(async move {
                let js_code = r#"
                    (function() {
                        const editor = document.getElementById('editor');
                        return editor?.innerText ?? '';
                    })();
                "#;
                
                if let Ok(js_result) = document::eval(js_code).await {
                    let new_text = js_result.as_str()
                        .unwrap_or_default()
                        .replace('\u{200b}', "");  // Remove zero-width spaces
                    
                    if !new_text.is_empty() && new_text != *raw_text.read() {
                        prev_raw_text.set(new_text.to_string());
                        raw_text.set(new_text.to_string());
                    }
                }
            });
        }
    };

    rsx! {
        div {
            style: "display: flex; gap: 20px; padding: 20px;",

            // Editor Pane: A contenteditable div showing the raw markdown text.
            div {
                style: "flex: 1; border: 1px solid #ccc; padding: 8px;",
                h3 { "Editor (Content Editable)" }
                // The contenteditable div uses an id ("editor") for JS interop.
                div {
                    id: "editor",
                    contenteditable: "true",
                    style: "height: 200px; overflow-y: auto; white-space: pre-wrap; border: 1px solid #aaa; padding: 8px;",
                    onkeydown: handle_keydown,
                    oninput: handle_input,
                    onmouseup: move |e| { update_caret_click(e); },
                    onclick: move |e| { update_caret_click(e); },
                    // Here we simply display the raw text.
                    // In a more advanced version you might run a syntax highlighter
                    // to wrap tokens in spans for color/styling.

                    { preview_nodes().clone().into_iter() }
          
                }
            },

            // Preview Pane: A read-only live preview rendered using MarkdownRenderer.
            div {
                id: "preview",
                
                style: "display: none; flex: 1; border: 1px solid #ccc; padding: 8px;",
                { preview_nodes().clone().into_iter() }
            }
        },
        // Debug info: show raw state and caret position.
        div {
            style: "margin-top: 1em; font-family: monospace;",
            "Raw Text: ",
            pre { "{raw_text}" },
            "Caret Position: ",
            {
            if let Some(pos) = *caret_pos.read() {
                rsx!{ "{pos}" }
            } else {
                rsx!{ "None" }
            }}
            br {},
            "Selection Range: ",
            {
                if let Some((start, end)) = *selection_range.read() {
                    rsx! { "Start: {start}, End: {end}" }
                } else {
                    rsx! { "None" }
                }
            }            
        }
    }
}
