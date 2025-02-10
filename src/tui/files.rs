//! Files list page

use crate::FileType;

use super::F;

use super::colors::{color_from_u8, get_style, FileColor};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Row, StatefulWidget, Table},
    Frame,
};

use std::fs;

pub struct FilesView<'a> {
    pub f: &'a mut F,
}

fn get_bytes_string(stream: Vec<u8>) -> String {
    let mut s = "".to_string();
    let mut i = 0;
    let mut cnt = 0;

    for chunk in stream {
        if cnt == 500 {
            break;
        }

        s.push_str(match chunk {
            0..10 => " 00",
            10..100 => " 0",
            _ => " ",
        });

        s.push_str(&chunk.to_string());

        i += 1;

        if i == 10 {
            s.push('\n');
            i = 0;
        }

        cnt += 1;
    }

    // Заполняем пропуски. Оставшиеся элементы (нули) - 10-i
    if i < 10 && i != 0 {
        i = 10 - i;
        s.push(' ');
    }

    for _ in 0..i {
        s.push_str("NUL ");
    }

    s
}

impl<'a> FilesView<'a> {
    fn panel_bytes(&self, area: Rect, frame: &mut Frame) {
        let preview_block = Block::bordered()
            .border_set(border::DOUBLE)
            .title(
                Line::from(" Bytes ")
                    .centered()
                    .bg(color_from_u8(self.f.colors.panels.header_bg).unwrap_or_default())
                    .fg(color_from_u8(self.f.colors.panels.header_fg).unwrap_or_default()),
            )
            .title_bottom("Show first 500 bytes")
            .style(
                Style::default().fg(color_from_u8(self.f.colors.panels.file).unwrap_or_default()),
            )
            .set_style(get_style(
                self.f.colors.panels.border_active,
                self.f.colors.panels.file_modifier,
            ));

        let view = Paragraph::new(match &self.f.selected {
            None => "-- Nothing to show --".to_string(),
            Some(selected) => {
                if selected.byte_size > 2_u64.pow(20) {
                    "-- File too large (> 1 MBytes) --".to_string()
                } else {
                    match selected.file_type {
                        //               text files may be executable
                        FileType::File | FileType::FileExecutable => {
                            match fs::read(&selected.path) {
                                Ok(stream) => {
                                    if stream.is_empty() {
                                        format!("-- Empty file --")
                                    } else {
                                        get_bytes_string(stream)
                                    }
                                }
                                Err(why) => format!("-- Failed to show file ({why}) --"),
                            }
                        }
                        _ => String::from("-- This file type doesn't supported to show --"),
                    }
                }
            }
        })
        .block(preview_block);

        frame.render_widget(view, area);
    }

    fn panel_preview(&self, area: Rect, frame: &mut Frame) {
        let preview_block = Block::bordered()
            .border_set(border::DOUBLE)
            .title(
                Line::from(" Preview ")
                    .centered()
                    .bg(color_from_u8(self.f.colors.panels.header_bg).unwrap_or_default())
                    .fg(color_from_u8(self.f.colors.panels.header_fg).unwrap_or_default()),
            )
            .style(
                Style::default().fg(color_from_u8(self.f.colors.panels.file).unwrap_or_default()),
            )
            .set_style(get_style(
                self.f.colors.panels.border_active,
                self.f.colors.panels.file_modifier,
            ));

        let view = Paragraph::new(match &self.f.selected {
            None => "-- Nothing to show --".to_string(),
            Some(selected) => {
                if selected.byte_size > 2_u64.pow(20) {
                    "-- File too large (> 1 MBytes) --".to_string()
                } else {
                    match selected.file_type {
                        //               text files may be executable
                        FileType::File | FileType::FileExecutable => {
                            match fs::read_to_string(&selected.path) {
                                Ok(string) => {
                                    if string.is_empty() {
                                        format!("-- Empty file --")
                                    } else {
                                        string
                                    }
                                }
                                Err(why) => format!("-- Failed to show file ({why}) --"),
                            }
                        }
                        _ => String::from("-- This file type doesn't supported to show --"),
                    }
                }
            }
        })
        .block(preview_block);

        frame.render_widget(view, area);
    }

    fn files_list(&mut self, area: Rect, frame: &mut Frame) {
        let mut files_block = Block::bordered()
            .border_set(border::DOUBLE)
            .style(
                Style::default().fg(color_from_u8(self.f.colors.panels.file).unwrap_or_default()),
            )
            .set_style(get_style(
                self.f.colors.panels.border_active,
                self.f.colors.panels.file_modifier,
            ))
            .title(match &self.f.selected {
                Some(selected) => format!(
                    " {} ({}/{}) ",
                    selected.file_name.to_string_lossy(),
                    self.f.idx.unwrap_or(0) + 1,
                    self.f.rows.len()
                ),
                None => String::new(),
            })
            .title_top(
                Line::from(format!(" {} files in this dir ", self.f.rows.len())).right_aligned(),
            )
            .title_top(
                Line::from(format!(
                    " {} ",
                    fs::canonicalize(&self.f.current_dir).unwrap().display()
                ))
                .centered()
                .bg(color_from_u8(self.f.colors.panels.header_bg).unwrap_or_default())
                .fg(color_from_u8(self.f.colors.panels.header_fg).unwrap_or_default()),
            );

        if self.f.show_hidden {
            files_block = files_block.title_bottom(" Show hidden files ON ");
        }

        if self.f.ts.selected().is_none() && !self.f.rows.is_empty() {
            self.f.ts.select(Some(0));
        }

        self.f
            .ts
            .selected()
            .and_then(|n| self.f.rows.get(n).cloned())
            .clone_into(&mut self.f.selected);

        let rows = self.f.rows.iter().map(|item| {
            let style = FileColor {
                entry: item,
                cols: self.f.colors.panels,
                selected: item.is_hidden,
            }
            .style();

            Row::new(vec![
                item.file_name
                    .to_string_lossy()
                    .to_string()
                    .set_style(style.clone()),
                item.file_type.to_string().set_style(style),
                item.size().to_string().into(),
            ])
        });
        let widths = [
            Constraint::Percentage(75),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
        ];

        let table = Table::new(rows, widths)
            .style(
                Style::new().bg(color_from_u8(self.f.colors.panels.background).unwrap_or_default()),
            )
            .row_highlight_style(
                Style::new()
                    .bg(color_from_u8(self.f.colors.panels.selection_color).unwrap_or_default()),
            )
            .block(files_block.clone());

        StatefulWidget::render(table, area, frame.buffer_mut(), &mut self.f.ts);
    }

    pub fn ui(&mut self, area: Rect, frame: &mut Frame) {
        if self.f.show_preview {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            self.files_list(chunks[0], frame);
            self.panel_preview(chunks[1], frame);
        } else if self.f.show_bytes {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            self.files_list(chunks[0], frame);
            self.panel_bytes(chunks[1], frame);
        } else {
            self.files_list(area, frame);
        }
    }
}
