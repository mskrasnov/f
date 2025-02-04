//! Experimental TUI for `f`

use crate::consts::{PROG_NAME, PROG_VER};
use crate::tui::colors::{color_from_u8, get_style, Colors, FileColor};
use crate::utils::get_home;
use crate::{traits::Toml, utils::read_dir, FileEntry, FileType};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Row, StatefulWidget, Table, TableState},
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

    fn ui(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(1)])
            .split(frame.area());

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
        frame.render_widget(title, chunks[1]);

        let fcols = self.colors.panels;
        let mut files_block = Block::bordered()
            .border_set(border::DOUBLE)
            .style(
                Style::default()
                    // .bg(color_from_u8(fcols.border_active).unwrap_or_default())
                    .fg(color_from_u8(fcols.file).unwrap_or_default()),
            )
            .set_style(get_style(fcols.border_active, fcols.file_modifier))
            .title(match &self.selected {
                Some(selected) => format!(
                    " {} ({}/{}) ",
                    selected.file_name.to_string_lossy(),
                    self.idx.unwrap_or(0) + 1,
                    self.rows.len()
                ),
                None => String::new(),
            })
            .title_top(
                Line::from(format!(" {} files in this dir ", self.rows.len())).right_aligned(),
            )
            .title_top(
                Line::from(format!(
                    " {} ",
                    fs::canonicalize(&self.current_dir).unwrap().display()
                ))
                .centered()
                .bg(color_from_u8(fcols.header_bg).unwrap_or_default())
                .fg(color_from_u8(fcols.header_fg).unwrap_or_default()),
            );

        if self.show_hidden {
            files_block = files_block.title_bottom(" Show hidden files ON ");
        }

        if self.ts.selected().is_none() && !self.rows.is_empty() {
            self.ts.select(Some(0));
        }

        self.ts
            .selected()
            .and_then(|n| self.rows.get(n).cloned())
            .clone_into(&mut self.selected);

        let max_size_fname_len: u16 = self
            .rows
            .iter()
            .map(|f| f.file_name.len())
            .max()
            .unwrap_or(0)
            .try_into()
            .unwrap_or(0);

        let rows = self.rows.iter().map(|item| {
            Row::new(vec![
                item.file_name.to_string_lossy().to_string().set_style(
                    FileColor {
                        entry: item,
                        cols: fcols,
                        selected: item.is_hidden,
                    }
                    .style(),
                ),
                item.file_type.to_string().into(),
                item.size().to_string().into(),
            ])
        });
        let widths = [
            Constraint::Min(max_size_fname_len),
            Constraint::Length(10),
            Constraint::Min(10),
        ];

        let table = Table::new(rows, widths)
            .style(Style::new().bg(color_from_u8(fcols.background).unwrap_or_default()))
            .row_highlight_style(
                Style::new().bg(color_from_u8(fcols.selection_color).unwrap_or_default()),
            )
            .block(files_block.clone());

        StatefulWidget::render(
            table,
            files_block.inner(frame.area()),
            frame.buffer_mut(),
            &mut self.ts,
        );
    }

    pub fn run(&mut self, term: &mut DefaultTerminal) -> Result<()> {
        while !self.is_exit {
            term.draw(|frame| self.ui(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
}
