use super::text::*;

#[derive(Debug)]
pub struct CellInfo {
    pub row: usize,
    pub col: usize,
    pub num_cols: usize,
    pub width: Option<f64>,
    pub syntax: MarkDownElements, // Include syntax field
}

#[derive(Debug)]
pub struct MarkDownStyle {
    pub text: String,
    pub font_size: usize,
    pub bold: bool,
    pub color: String,
    pub width: String,
    pub flex_grow: i32
}

pub fn compute_markdown_style_props(props: CellInfo) -> MarkDownStyle {
    // Extract text from MarkDownElements
    let text = match props.syntax.clone() {
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
    };

    let width = if let Some(w) = props.width {
        format!("{}px", w)
    }
    else {
        "auto".to_string()
    };
    // let width = props.width.map_or("auto".to_string(), |w| format!("{}px", w));

    // Set font size based on heading level or text type
    let font_size = match props.syntax.clone() {
        MarkDownElements::Heading(heading) => match heading {
            HeadingLevel::H1(_) => 32, // H1: 32px
            HeadingLevel::H2(_) => 28, // H2: 28px
            HeadingLevel::H3(_) => 24, // H3: 24px
            HeadingLevel::H4(_) => 20, // H4: 20px
            HeadingLevel::H5(_) => 18, // H5: 18px
            HeadingLevel::H6(_) => 16, // H6: 16px
        },
        MarkDownElements::PlainText(_) => 16, // Plain text: 16px
        MarkDownElements::BoldText(_) => 16,  // Bold text: 16px
        MarkDownElements::EmptySpace => 16
    };

    // Set bold: headings and bold text are bold
    let bold = matches!(
        props.syntax,
        MarkDownElements::Heading(_) | MarkDownElements::BoldText(_)
    );

    // Color is always black
    let color = "black".to_string();

    // let mut flex_grow = 0;
    // Set width and flex-grow: last column uses flex-grow: 1
    let flex_grow = if props.col == props.num_cols - 1 {
        1

        // format!("flex-grow: 1; width: {};", props.width)
    } else {
        // format!("flex-grow: 0; width: {};", props.width)
        0
    };

    MarkDownStyle {
        text,
        font_size,
        bold,
        color,
        width,
        flex_grow
    }
}

pub fn compute_markdown_style_string(props: MarkDownStyle) -> String {
    format!(
        "font-size: {}px; color: {}; font-weight: {}; width: {}px; flex-grow: {}",
        props.font_size,
        props.color,
        if props.bold { "bold" } else { "normal" },
        props.width,
        props.flex_grow
    )
}