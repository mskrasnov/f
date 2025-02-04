//! Color schemes for `f`
//!
//! ## Color table
//!
//! | Name           | FG code | BG code |
//! |----------------|---------|---------|
//! | Black          | 30      | 40      |
//! | Red            | 31      | 41      |
//! | Green          | 32      | 42      |
//! | Yellow         | 33      | 43      |
//! | Blue           | 34      | 44      |
//! | Magenta        | 35      | 45      |
//! | Cyan           | 36      | 46      |
//! | White          | 37      | 47      |
//! | Grey           | 38      | 48      |
//! | Light Red      | 91      | 101     |
//! | Light Green    | 92      | 102     |
//! | Light Yellow   | 93      | 103     |
//! | Light Blue     | 94      | 104     |
//! | Light Magenta  | 95      | 105     |
//! | Light Cyan     | 96      | 106     |
//! | Dark Gray      | 98      | 108     |
//!
//! ## Modifiers
//!
//! | Name      | Code |
//! |-----------|------|
//! | Bold      | 1    |
//! | Italic    | 2    |
//! | Underline | 3    |
//! | B&I       | 4    |
//! | B&U       | 5    |
//! | I&U       | 6    |
//! | B&I&U     | 7    |
//! | None      | 8    |

use core::str;

use crate::traits::Toml;
use ratatui::style::{Color, Style, Stylize};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone, Copy)]
pub struct Colors {
    pub title: Title,
    pub panels: Panels,
    pub footer: Footer,
}

impl Toml for Colors {}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct Title {
    pub background: u8,
    pub text: u8,
    pub text_modifier: Option<u8>,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct Panels {
    pub background: u8,

    pub border_active: u8,
    pub border_inactive: u8,

    pub file: u8,
    pub file_selected: Option<u8>,
    pub file_modifier: Option<u8>,

    pub exec_file: u8,
    pub exec_file_selected: Option<u8>,
    pub exec_file_modifier: Option<u8>,

    pub link: u8,
    pub link_selected: Option<u8>,
    pub link_modifier: Option<u8>,

    pub special_file: u8,
    pub special_file_selected: Option<u8>,
    pub special_file_modifier: Option<u8>,

    pub dir: u8,
    pub dir_selected: Option<u8>,
    pub dir_modifier: Option<u8>,

    pub selection_color: u8,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct Footer {
    pub key_code: u8,
    pub key_code_modifier: Option<u8>,
    pub key_title: u8,
    pub key_title_modifier: Option<u8>,
    pub background: u8,
}

pub fn color_from_u8(color: u8) -> Option<Color> {
    match color {
        30 | 40 => Some(Color::Black),
        31 | 41 => Some(Color::Red),
        32 | 42 => Some(Color::Green),
        33 | 43 => Some(Color::Yellow),
        34 | 44 => Some(Color::Blue),
        35 | 45 => Some(Color::Magenta),
        36 | 46 => Some(Color::Cyan),
        37 | 47 => Some(Color::White),
        38 | 48 => Some(Color::Gray),

        91 | 101 => Some(Color::LightRed),
        92 | 102 => Some(Color::LightGreen),
        93 | 103 => Some(Color::LightYellow),
        94 | 104 => Some(Color::LightBlue),
        95 | 105 => Some(Color::LightMagenta),
        96 | 106 => Some(Color::LightCyan),

        98 | 108 => Some(Color::DarkGray),

        _ => None,
    }
}

pub enum Modifier {
    Bold,
    Italic,
    Underline,
    BoldItalic,
    BoldUnderline,
    ItalicUnderline,
    BoldItalicUnderline,
    None,
}

impl From<u8> for Modifier {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Bold,
            2 => Self::Italic,
            3 => Self::Underline,
            4 => Self::BoldItalic,
            5 => Self::BoldUnderline,
            6 => Self::ItalicUnderline,
            7 => Self::BoldItalicUnderline,
            _ => Self::None,
        }
    }
}

pub fn get_style(color_code: u8, modifier: Option<u8>) -> Style {
    let color = color_from_u8(color_code).unwrap_or(Color::Reset);
    let mut style = Style::default();

    if color_code >= 40 {
        style = style.bg(color);
    } else {
        style = style.fg(color);
    }

    let modifier = match modifier {
        Some(m) => Modifier::from(m),
        None => Modifier::None,
    };

    match modifier {
        Modifier::Bold => style = style.bold(),
        Modifier::Italic => style = style.italic(),
        Modifier::Underline => style = style.underlined(),
        Modifier::BoldItalic => style = style.bold().italic(),
        Modifier::BoldUnderline => style = style.bold().underlined(),
        Modifier::ItalicUnderline => style = style.italic().underlined(),
        Modifier::BoldItalicUnderline => style = style.bold().italic().underlined(),
        _ => {}
    }

    style
}

impl Default for Title {
    fn default() -> Self {
        Self {
            background: 30,
            text: 93,
            text_modifier: Some(1),
        }
    }
}

impl Default for Panels {
    fn default() -> Self {
        Self {
            background: 98,
            border_active: 34,
            border_inactive: 30,

            file: 37,
            file_selected: Some(30),
            file_modifier: None,

            exec_file: 32,
            exec_file_selected: Some(30),
            exec_file_modifier: None,

            link: 32,
            link_selected: Some(33),
            link_modifier: None,

            special_file: 36,
            special_file_selected: None,
            special_file_modifier: None,

            dir: 34,
            dir_selected: None,
            // dir_modifier: Some(1),
            dir_modifier: None,

            selection_color: 96,
        }
    }
}

impl Default for Footer {
    fn default() -> Self {
        Self {
            key_code: 31,
            key_code_modifier: Some(1),
            key_title: 30,
            key_title_modifier: None,
            background: 38,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conf_write_test() {
        let pth = "./colors.toml";
        Colors::default().write(pth).unwrap();
    }
}
