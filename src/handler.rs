pub fn handle_backspace_no_selection(read_text: &str, pos: usize) -> (String, usize) {
    if pos > 0 && pos < read_text.len() && &read_text[pos - 1..pos + 1] == "\r\n" {
        // Case: Caret is between `\r\n`, delete both
        let new_text = format!("{}{}", &read_text[..pos - 1], &read_text[pos + 1..]);
        return (new_text, pos - 1);
    } else if pos >= 2 && &read_text[pos - 2..pos] == "\r\n" {
        // Case: Caret is after `\n` and `\r\n` exists before it, delete both
        let new_text = format!("{}{}", &read_text[..pos - 2], &read_text[pos..]);
        return (new_text, pos - 2);
    } else if pos > 0 {
        // Case: Delete a single character
        let mut chars: Vec<char> = read_text.chars().collect();
        chars.remove(pos - 1);
        return (chars.into_iter().collect(), pos - 1);
    }

    // If pos is 0 or invalid, return original text
    (read_text.to_string(), pos)
}

pub fn handle_character_input(read_text: &str, pos: usize, ch: &String) -> (String, usize) {
    let (left, right) = read_text.split_at(pos);
    
    // If inserting a space and the left side ends with a space, convert to &nbsp;
    let new_char = if ch == " " && left.ends_with(' ') {
        " ".to_string()
    } else {
        ch.to_string()
    };

    let new_text = format!("{}{}{}", left, new_char, right);
    let updated_pos = pos + new_char.len(); // Move caret correctly

    (new_text, updated_pos)
}


pub fn handle_enter_key(read_text: &str, pos: usize, last_key_was_enter: bool) -> (String, usize) {
    let (left, right) = read_text.split_at(pos);

    // Do not trim aggressively; only normalize existing Windows-style newlines
    let left_trimmed = left.strip_suffix("\r").unwrap_or(left); // Strip only if it's \r\n
    let right_trimmed = right.strip_prefix("\n").unwrap_or(right); // Strip only if it's \n\r

    let new_text = if last_key_was_enter {
        // If the previous key was also Enter, insert two new lines to create a blank paragraph
        format!("{}\r\n{}", left_trimmed, right_trimmed)
    } else {
        // Otherwise, just insert a single `\n\r`
        format!("{}\r\n{}", left_trimmed, right_trimmed)
    };

    // Move caret position accordingly
    let updated_pos = left_trimmed.len() + if last_key_was_enter { 2 } else { 2 };

    (new_text, updated_pos)
}

