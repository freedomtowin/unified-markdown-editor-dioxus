static NEWLINE_PLACEHOLDER: &str = "__LITERAL_NEWLINE__";

#[derive(Debug, PartialEq, Clone)]
pub enum HeadingLevel {
    H1(String),
    H2(String),
    H3(String),
    H4(String),
    H5(String),
    H6(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum MarkDownElements {
    Heading(HeadingLevel),
    PlainText(String),
    BoldText(String),
    EmptySpace
}

pub struct TextProcessor {
    language: String,
}

impl TextProcessor {
    pub fn new() -> Self {
        Self {
            language: "markdown".to_string(),
        }
    }

    /// Processes markdown text into a Vec<Vec<MarkDownElements>> where:
    /// - Outer Vec represents rows (newlines or new blocks like headers)
    /// - Inner Vec contains MarkDownElements enums (Heading, PlainText, BoldText)
    /// - Text content retains original markdown formatting (e.g., #, **, __)
    pub fn process_markdown(&self, input: String) -> Vec<Vec<MarkDownElements>> {
        // Standardize input
        let temp = input.replace("\\n", NEWLINE_PLACEHOLDER);
        let without_tabs = temp.replace("\t", "    ");
        let standardized_newlines = without_tabs.replace("\r\n", "\n").replace("\r", "\n");
    
        // Split into lines
        let lines: Vec<&str> = standardized_newlines.lines().collect();
        let mut result: Vec<Vec<MarkDownElements>> = Vec::new();
        let mut empty_count = 0;
    
        for line in lines {
            // Handle empty lines
            if line.trim().is_empty() {
                empty_count += 1;
                continue;
            }
    
            while empty_count > 0 {
                result.push(vec![MarkDownElements::EmptySpace]);
                empty_count -= 1;
            }
    
            let mut row: Vec<MarkDownElements> = Vec::new();
    
            // Check for headers (#, ##, ###, etc.)
            let trimmed = line.trim_start();
            let header_level = line.chars().take_while(|c| *c == '#').count();
            if header_level > 0 && header_level <= 6 && trimmed.starts_with('#') && trimmed.len() > header_level {
                if trimmed.chars().nth(header_level) == Some(' ') {
                    let header_text = format!("#{} {}", &"#".repeat(header_level - 1), trimmed[header_level + 1..].to_string());
                    let heading = match header_level {
                        1 => HeadingLevel::H1(header_text),
                        2 => HeadingLevel::H2(header_text),
                        3 => HeadingLevel::H3(header_text),
                        4 => HeadingLevel::H4(header_text),
                        5 => HeadingLevel::H5(header_text),
                        6 => HeadingLevel::H6(header_text),
                        _ => unreachable!(),
                    };
                    row.push(MarkDownElements::Heading(heading));
                    result.push(row);
                    continue;
                }
            }
    
            // Process inline elements (bold and plain text)
            let mut current = line;
            while !current.is_empty() {
                // Check for bold (**text** or __text__)
                let bold_patterns = vec![("**", "**"), ("__", "__")];
                let mut found_bold = false;
    
                for (open, close) in bold_patterns.clone() {
                    if current.starts_with(open) {
                        // Find closing delimiter
                        if let Some(end_idx) = current[open.len()..].find(close) {
                            let bold_text = &current[..open.len() + end_idx + close.len()];
                            if bold_text.len() > open.len() + close.len() {
                                row.push(MarkDownElements::BoldText(bold_text.to_string()));
                                // Move past the bold text
                                let bold_end = open.len() + end_idx + close.len();
                                // Check for trailing spaces after bold text
                                let trailing = &current[bold_end..];
                                let trailing_spaces = trailing.chars().take_while(|c| c.is_whitespace()).count();
                                if trailing_spaces > 0 {
                                    row.push(MarkDownElements::PlainText(" ".repeat(trailing_spaces)));
                                    current = &current[bold_end + trailing_spaces..];
                                } else {
                                    current = &current[bold_end..];
                                }
                                found_bold = true;
                                break;
                            }
                        }
                    }
                }
    
                if !found_bold {
                    // Extract plain text until next bold marker or end
                    let next_bold = bold_patterns
                        .iter()
                        .filter_map(|(open, _)| current.find(open).map(|idx| (idx, open)))
                        .min_by_key(|&(idx, _)| idx);
    
                    match next_bold {
                        Some((idx, _)) if idx > 0 => {
                            let plain_text = current[..idx].to_string();
                            if !plain_text.trim().is_empty() {
                                row.push(MarkDownElements::PlainText(plain_text));
                            }
                            current = &current[idx..];
                        }
                        _ => {
                            let plain_text = current.to_string();
                            if !plain_text.trim().is_empty() {
                                row.push(MarkDownElements::PlainText(plain_text));
                            }
                            current = "";
                        }
                    }
                }
            }
    
            if !row.is_empty() {
                result.push(row);
            }
        }
    
        while empty_count > 0 {
            result.push(vec![MarkDownElements::EmptySpace]);
            empty_count -= 1;
        }
        result.push(vec![MarkDownElements::EmptySpace]);
        result
    }

    pub fn process_text(&self, input: String) -> Vec<Vec<String>> {
        // Standardize input
        let temp = input.replace("\\n", NEWLINE_PLACEHOLDER);
        let without_tabs = temp.replace("\t", "    ");
        let standardized_newlines = without_tabs.replace("\r\n", "\n").replace("\r", "\n");
        
        // Split into lines
        let lines: Vec<&str> = standardized_newlines.lines().collect();
        let mut result: Vec<Vec<String>> = Vec::new();

        for line in lines {
            let mut row: Vec<String> = Vec::new();
            
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Check for headers (#, ##, ###, etc.)
            let trimmed = line.trim_start();
            let header_level = line.chars().take_while(|c| *c == '#').count();
            if header_level > 0 && trimmed.starts_with('#') && trimmed.len() > header_level {
                // Ensure there's a space after # symbols
                if trimmed.chars().nth(header_level) == Some(' ') {
                    let header_text = trimmed[header_level + 1..].trim().to_string();
                    if !header_text.is_empty() {
                        row.push(format!("header{}|{}", header_level, header_text));
                        result.push(row);
                        continue;
                    }
                }
            }

            // Process inline elements (bold and plain text)
            let mut current = line;
            while !current.is_empty() {
                // Check for bold (**text** or __text__)
                let bold_patterns = vec![("**", "**"), ("__", "__")];
                let mut found_bold = false;

                for (open, close) in bold_patterns.clone() {
                    if current.starts_with(open) {
                        // Find closing delimiter
                        if let Some(end_idx) = current[open.len()..].find(close) {
                            let bold_text = &current[open.len()..open.len() + end_idx];
                            if !bold_text.is_empty() {
                                row.push(format!("bold|{}", bold_text));
                                current = &current[open.len() + end_idx + close.len()..];
                                found_bold = true;
                                break;
                            }
                        }
                    }
                }

                if !found_bold {
                    // Extract plain text until next bold marker or end
                    let next_bold = bold_patterns
                        .iter()
                        .filter_map(|(open, _)| current.find(open).map(|idx| (idx, open)))
                        .min_by_key(|&(idx, _)| idx);

                    match next_bold {
                        Some((idx, _)) if idx > 0 => {
                            let plain_text = current[..idx].trim();
                            if !plain_text.is_empty() {
                                row.push(format!("plain|{}", plain_text));
                            }
                            current = &current[idx..];
                        }
                        _ => {
                            let plain_text = current.trim();
                            if !plain_text.is_empty() {
                                row.push(format!("plain|{}", plain_text));
                            }
                            current = "";
                        }
                    }
                }
            }

            if !row.is_empty() {
                result.push(row);
            }
        }

        result
    }

    pub fn extract_strings(&self, input: Vec<Vec<MarkDownElements>>) -> Vec<Vec<String>> {
        input.into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|element| match element {
                        MarkDownElements::Heading(heading) => match heading {
                            HeadingLevel::H1(text) => text,
                            HeadingLevel::H2(text) => text,
                            HeadingLevel::H3(text) => text,
                            HeadingLevel::H4(text) => text,
                            HeadingLevel::H5(text) => text,
                            HeadingLevel::H6(text) => text,
                        },
                        MarkDownElements::PlainText(text) => text,
                        MarkDownElements::BoldText(text) => text,
                        MarkDownElements::EmptySpace => "".to_string()
                    })
                    .collect::<Vec<String>>()
            })
            .collect::<Vec<Vec<String>>>()
    }

    pub fn markdown_to_string(&self, input: Vec<Vec<String>>) -> String {
        input.into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|text| text)
                    .collect::<Vec<String>>()
                    .join("")
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_markdown() {
        let processor = TextProcessor::new();
        let input = "# Header\nNormal text **bold text** more text\n## Subheader\n__another bold__";
        let result = processor.process_markdown(input.to_string());

        let expected = vec![
            vec![MarkDownElements::Heading(HeadingLevel::H1("# Header".to_string()))],
            vec![
                MarkDownElements::PlainText("Normal text ".to_string()),
                MarkDownElements::BoldText("**bold text**".to_string()),
                MarkDownElements::PlainText(" more text".to_string()),
            ],
            vec![MarkDownElements::Heading(HeadingLevel::H2("## Subheader".to_string()))],
            vec![MarkDownElements::BoldText("__another bold__".to_string())],
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_lines_and_invalid_headers() {
        let processor = TextProcessor::new();
        let input = "# Valid Header\n\nInvalid#Header\n**bold**";
        let result = processor.process_markdown(input.to_string());

        let expected = vec![
            vec![MarkDownElements::Heading(HeadingLevel::H1("# Valid Header".to_string()))],
            vec![MarkDownElements::PlainText("Invalid#Header".to_string())],
            vec![MarkDownElements::BoldText("**bold**".to_string())],
        ];

        assert_eq!(result, expected);
    }
}


pub fn process_text(input: String) -> Vec<Vec<String>> {
    // Standardize input
    let temp = input.replace("\\n", NEWLINE_PLACEHOLDER);
    let without_tabs = temp.replace("\t", "    ");
    let standardized_newlines = without_tabs.replace("\r\n", "\n").replace("\r", "\n");
    
    // Split into lines
    let lines: Vec<&str> = standardized_newlines.lines().collect();
    let mut result: Vec<Vec<String>> = Vec::new();

    for line in lines {
        let mut row: Vec<String> = Vec::new();
        
        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Check for headers (#, ##, ###, etc.)
        let trimmed = line.trim_start();
        let header_level = line.chars().take_while(|c| *c == '#').count();
        if header_level > 0 && trimmed.starts_with('#') && trimmed.len() > header_level {
            // Ensure there's a space after # symbols
            if trimmed.chars().nth(header_level) == Some(' ') {
                let header_text = trimmed[header_level + 1..].trim().to_string();
                if !header_text.is_empty() {
                    row.push(format!("header{}|{}", header_level, header_text));
                    result.push(row);
                    continue;
                }
            }
        }

        // Process inline elements (bold and plain text)
        let mut current = line;
        while !current.is_empty() {
            // Check for bold (**text** or __text__)
            let bold_patterns = vec![("**", "**"), ("__", "__")];
            let mut found_bold = false;

            for (open, close) in bold_patterns.clone() {
                if current.starts_with(open) {
                    // Find closing delimiter
                    if let Some(end_idx) = current[open.len()..].find(close) {
                        let bold_text = &current[open.len()..open.len() + end_idx];
                        if !bold_text.is_empty() {
                            row.push(format!("bold|{}", bold_text));
                            current = &current[open.len() + end_idx + close.len()..];
                            found_bold = true;
                            break;
                        }
                    }
                }
            }

            if !found_bold {
                // Extract plain text until next bold marker or end
                let next_bold = bold_patterns
                    .iter()
                    .filter_map(|(open, _)| current.find(open).map(|idx| (idx, open)))
                    .min_by_key(|&(idx, _)| idx);

                match next_bold {
                    Some((idx, _)) if idx > 0 => {
                        let plain_text = current[..idx].trim();
                        if !plain_text.is_empty() {
                            row.push(format!("plain|{}", plain_text));
                        }
                        current = &current[idx..];
                    }
                    _ => {
                        let plain_text = current.trim();
                        if !plain_text.is_empty() {
                            row.push(format!("plain|{}", plain_text));
                        }
                        current = "";
                    }
                }
            }
        }

        if !row.is_empty() {
            result.push(row);
        }
    }

    result
}