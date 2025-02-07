//! Experimental TUI for `f`

use crate::consts::{PROG_NAME, PROG_VER};
use crate::tui::colors::{color_from_u8, get_style, Colors, FileColor};
use crate::utils::get_home;
use crate::{traits::Toml, utils::read_dir, FileEntry, FileType};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Row, StatefulWidget, Table, TableState},
    DefaultTerminal, Frame,
};

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};

/// Main `f` TUI
pub struct F {
    colors: Colors,
    show_hidden: bool,
    show_preview: bool,
    error_text: Option<String>,

    ts: TableState,
    current_dir: PathBuf,
    rows: Vec<FileEntry>,
    selected: Option<FileEntry>,
    idx: Option<usize>,

    is_exit: bool,
}

impl F {
    pub fn new<P: AsRef<Path>>(pth: P) -> Result<Self> {
        let rows = read_dir(&pth, false)?;

        Ok(Self {
            current_dir: pth.as_ref().to_path_buf(),
            colors: Colors::parse("./colors.toml").unwrap_or_default(),
            ts: TableState::default(),
            selected: if rows.len() > 0 {
                Some(rows[0].clone())
            } else {
                None
            },
            rows,
            idx: None,
            error_text: None,
            show_hidden: false,
            show_preview: true,

            is_exit: false,
        })
    }

    fn rescan_dir(&mut self) -> Result<()> {
        self.rows = read_dir(&self.current_dir, self.show_hidden)?;
        if self.rows.len() > 0 {
            self.idx = Some(0);
            self.ts.select(self.idx);
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.is_exit = true;
    }

    fn remove_error_msg(&mut self) {
        self.error_text = None;
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                self.handle_key_event(key);
            }
            _ => {}
        };
        Ok(())
    }

    fn update_idx(&mut self) {
        self.rows
            .get(self.ts.selected().unwrap_or(0))
            .cloned()
            .clone_into(&mut self.selected);
        self.idx = Some(self.ts.selected().unwrap_or(0));
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::F(8) => {
                if let Some(selected) = &self.selected {
                    if selected.file_name == OsString::from_str(".. [UP]").unwrap()
                        || self.idx == Some(0)
                    {
                        self.error_text = Some("Failed to remove parent directory".to_string());
                    } else {
                        if let Err(why) = selected.remove() {
                            self.error_text = Some(why.to_string());
                        } else {
                            if let Err(why) = self.rescan_dir() {
                                self.error_text = Some(why.to_string());
                            }
                        }
                    }
                }
            }
            KeyCode::Delete => {
                if let Some(selected) = &self.selected {
                    if selected.file_name == OsString::from_str(".. [UP]").unwrap()
                        || self.idx == Some(0)
                    {
                        self.error_text = Some("Failed to remove parent directory".to_string());
                    } else {
                        if let Err(why) = selected.remove_bin() {
                            self.error_text = Some(why.to_string());
                        } else {
                            if let Err(why) = self.rescan_dir() {
                                self.error_text = Some(why.to_string());
                            }
                        }
                    }
                }
            }
            KeyCode::F(10) | KeyCode::Char('q') | KeyCode::Char('й') => {
                self.exit();
            }

            KeyCode::Down | KeyCode::Char('j') => {
                if self.idx.unwrap_or(0) < (self.rows.len() - 1) {
                    self.ts.select_next();
                    self.update_idx();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.ts.select_previous();
                self.update_idx();
            }
            KeyCode::Home | KeyCode::Char('H') => {
                if self.rows.len() > 0 {
                    self.idx = Some(0);
                    self.ts.select(self.idx);
                }
            }
            KeyCode::End | KeyCode::Char('L') => {
                if self.rows.len() > 0 {
                    self.idx = Some(self.rows.len() - 1);
                    self.ts.select(self.idx);
                }
            }
            KeyCode::Char('~') => {
                let pth = get_home();
                self.selected = Some(FileEntry {
                    file_name: OsString::from_str("~").unwrap(),
                    path: pth.clone(),
                    byte_size: 4096,
                    file_type: FileType::Directory,
                    is_hidden: false,
                });
                self.current_dir = pth;

                if let Err(why) = self.rescan_dir() {
                    self.error_text = Some(why.to_string());
                }
            }
            KeyCode::Char('/') => {
                let pth = Path::new("/").to_path_buf();
                self.selected = Some(FileEntry {
                    file_name: OsString::from_str("~").unwrap(),
                    path: pth.clone(),
                    byte_size: 4096,
                    file_type: FileType::Directory,
                    is_hidden: false,
                });
                self.current_dir = pth;

                if let Err(why) = self.rescan_dir() {
                    self.error_text = Some(why.to_string());
                }
            }
            KeyCode::Char('.') => {
                self.show_hidden = !self.show_hidden;
                if let Err(why) = self.rescan_dir() {
                    self.error_text = Some(why.to_string());
                }
            }
            KeyCode::Char('p') => {
                self.show_preview = !self.show_preview;
            }

            KeyCode::Enter => {
                self.remove_error_msg();
                if let Some(selected) = self.selected.clone() {
                    // Сохраняем путь до предыдущей текущей директории чтобы восстановить его
                    // в случае ошибки (например, когда не можем зайти в новую директорию)
                    let old_cur_dir = self.current_dir.clone();

                    if selected.path.is_dir() {
                        self.current_dir = selected.path;
                        if let Err(why) = self.rescan_dir() {
                            self.current_dir = old_cur_dir; // восстанавливаем старый путь
                            self.error_text = Some(why.to_string());
                        }
                    }
                }
            }

            KeyCode::Esc => self.remove_error_msg(),
            KeyCode::Char('c') => self.update_colors(),
            _ => {}
        }
    }

    fn update_colors(&mut self) {
        match Colors::parse("./colors.toml") {
            Ok(colors) => self.colors = colors,
            Err(why) => self.error_text = Some(why.to_string()),
        }
    }

    fn keys(&self) -> Line<'_> {
        Line::from(vec![
            "F8".bold().red(),
            " Force delete  ".into(),
            "Del".bold().red(),
            " Safe delete  ".into(),
            "~".bold().red(),
            " Go home  ".into(),
            "/".bold().red(),
            " Go root  ".into(),
            ".".bold().red(),
            " Show hidden  ".into(),
            "p".bold().red(),
            " Show preview  ".into(),
            "q".bold().red(),
            " Quit".into(),
        ])
    }

    fn ui(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(5),
                Constraint::Length(1),
            ])
            .split(frame.area());

        frame.render_widget(self.keys(), chunks[2]);

        let tcols = self.colors.title;
        let title = match &self.error_text {
            None => Line::from(format!("The {} file manager ver.{}", PROG_NAME, PROG_VER))
                .set_style(get_style(tcols.background, tcols.text_modifier))
                .fg(color_from_u8(tcols.text).unwrap_or_default())
                .bg(color_from_u8(tcols.background).unwrap_or_default())
                .centered(),
            Some(text) => Line::from(format!("Error: {text} (press <Esc> to close)"))
                .set_style(get_style(tcols.background, tcols.text_modifier))
                .fg(Color::Red)
                .centered(),
        };
        frame.render_widget(title, chunks[0]);

        let mut ui = FilesView { f: self };
        ui.ui(chunks[1], frame);
    }

    pub fn run(&mut self, term: &mut DefaultTerminal) -> Result<()> {
        while !self.is_exit {
            term.draw(|frame| self.ui(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
}

pub struct FilesView<'a> {
    f: &'a mut F,
}

impl<'a> FilesView<'a> {
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
                if selected.byte_size > 2_u64.pow(20) * 10 {
                    "-- File too large (> 10 MBytes) --".to_string()
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

        let max_size_fname_len: u16 = self
            .f
            .rows
            .iter()
            .map(|f| f.file_name.len())
            .max()
            .unwrap_or(0)
            .try_into()
            .unwrap_or(0);

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
            Constraint::Min(max_size_fname_len),
            Constraint::Length(10),
            Constraint::Min(10),
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
        } else {
            self.files_list(area, frame);
        }
    }
}
