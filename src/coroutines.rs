use dioxus::prelude::*;
use crate::builder::EditorBuilder;
use crate::get_element_id;
use std::time::Duration;

use base64::{engine::general_purpose, Engine as _};

pub async fn measure_width_coroutine(mut rx: UnboundedReceiver<(usize, usize, String)>, editor: &mut Signal<EditorBuilder>) {

    println!("[measure widths] coroutine is running");

    loop {
        // Try to receive a message
        match rx.try_next() {
            Ok(Some((index_i, index_j, text))) => {

                let js = format!(
                    r#"
    
                    let span = document.createElement('span');
                    span.style.fontFamily = 'CqMono';
                    span.style.fontSize = '16px';
                    span.style.padding = '0px';
                    span.style.lineHeight = '1.2';
                    span.style.position = 'absolute';
                    span.style.visibility = 'hidden';
                    span.style.whiteSpace = 'pre';
                    span.textContent = '{}';
                    document.body.appendChild(span);
                    let width = span.offsetWidth + 0; // Add 2px padding * 2 + 1px border * 2
                    document.body.removeChild(span);
                    return width;
    
                    "#,
                    text
                );
    
                if let Ok(result) = document::eval(&js).await {
                    
                    let maybe_width = result.as_f64();
    
                    // let flat_index = editor.read().raw_text.iter().take(index_i).map(|inner| inner.len()).sum::<usize>() + index_j;

                    // let row_size = editor.read().text_width.len();

                    while index_i >= editor.read().text_width.len() {
                        // editor.write().text_width
                        editor.write().text_width.push(vec![None]);
                    }

                    while index_j >= editor.read().text_width[index_i].len() {
                        // editor.write().text_width
                        editor.write().text_width[index_i].push(None);
                    }

                    editor.write().text_width[index_i][index_j] = maybe_width;

                }
            }
            Ok(None) => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}


pub async fn focus_caret_position_coroutine(mut rx: UnboundedReceiver<(usize, usize)>, editor: &mut Signal<EditorBuilder>) {

    loop {
        // Try to receive a message
        match rx.try_next() {
            Ok(Some((index_i, index_j))) => {

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
                            println!("Caret position for index {} {}: {}", index_i, index_j, pos);
                            editor.with_mut(|e| e.move_caret(index_i, index_j, pos));
                        } else {
                            eprintln!("Failed to parse caret position: {:?}", result);
                        }
                    }
                    Err(e) => {
                        eprintln!("JS eval error: {}", e);
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

}



pub async fn update_editor_text_coroutine(mut rx: UnboundedReceiver<(usize, usize, usize, String)>, editor: &mut Signal<EditorBuilder>) {

    loop {
        // Try to receive a message
        match rx.try_next() {
            Ok(Some((index_i, index_j, cursor_pos, new_text))) => {
                    // tokio::time::sleep(Duration::from_millis(1)).await;

                    let element_id = get_element_id(index_i, index_j);
                    let text_b64 = general_purpose::STANDARD.encode(new_text.clone());
                
                    let js = format!(
                        r#"
                        return window.clearElementTextWithPosition('{}', atob("{}"), {});
                        "#,
                        element_id,
                        text_b64,
                        cursor_pos
                    );
        
                    let _ = document::eval(&js).await;

                    editor.write().update_text(index_i, index_j, new_text.clone());
        
                    editor.write().move_caret(index_i, index_j , cursor_pos);

                    println!("[update_editor_text_coroutine] index i {} index j {} cursor {} text {}", index_i, index_j, cursor_pos, new_text.clone());
            }
            Ok(None) => {
                // No messages available, wait for a short time before trying again
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(1)).await;
                // Channel is closed, exit the loop
                // println!("Channel closed");
                // break;
            }
        }
    }


}