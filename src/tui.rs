//! Text user interface for `f`

pub mod colors;

use crate::consts::{PROG_NAME, PROG_VER};
use crate::utils::get_home;
use crate::{traits::Toml, FileEntry, FileType};
use colors::{color_from_u8, get_style, Colors};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::Direction;
use ratatui::symbols::border;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style, Styled, Stylize},
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

/// Get path to the parent (upper) directory
fn up_dir<P: AsRef<Path>>(current_dir: P) -> FileEntry {
    FileEntry {
        file_name: OsString::from_str(".. [UP]").unwrap(),
        path: fs::canonicalize(current_dir.as_ref())
            .unwrap_or(Path::new("/tmp").to_path_buf())
            .parent()
            .unwrap_or(Path::new("/"))
            .to_path_buf(),
        byte_size: 4096,
        is_hidden: false,
        file_type: FileType::ParentDirectory,
    }
}

fn get_file_color(f: &FileEntry, cols: colors::Panels) -> Color {
    if !f.is_hidden {
        match f.file_type {
            FileType::Directory | FileType::ParentDirectory => color_from_u8(cols.dir),
            FileType::Link => color_from_u8(cols.link),
            FileType::Special => color_from_u8(cols.special_file),
            _ => color_from_u8(cols.file),
        }
        .unwrap_or_default()
    } else {
        match f.file_type {
            FileType::Directory | FileType::ParentDirectory => color_from_u8(cols.dir + 60),
            FileType::Link => color_from_u8(cols.link + 60),
            FileType::Special => color_from_u8(cols.special_file + 60),
            _ => color_from_u8(cols.file + 60),
        }
        .unwrap_or_default()
    }
}

/// Main `f` interface
pub struct F {
    colors: Colors,
    ts: TableState,
    current_dir: PathBuf,
    rows: Vec<FileEntry>,
    selected: Option<FileEntry>,
    idx: Option<usize>,
    error_text: Option<String>,
    show_hidden: bool,

    is_exit: bool,
}

impl F {
    pub fn new<P: AsRef<Path>>(pth: P) -> Result<Self> {
        Ok(Self {
            current_dir: pth.as_ref().to_path_buf(),
            colors: Colors::parse("./colors.toml").unwrap_or_default(),
            ts: TableState::default(),
            rows: {
                let dir = fs::read_dir(&pth)?;
                let mut rows = dir
                    .map(|entry| FileEntry::from_dir_entry(&entry.unwrap()).unwrap())
                    .collect::<Vec<_>>();
                if pth.as_ref() != Path::new("/") {
                    rows.insert(0, up_dir(&pth))
                };
                rows.sort_by_key(|key| key.file_name.clone());
                rows = rows
                    .iter()
                    .filter_map(|entry| {
                        if entry.is_hidden {
                            None
                        } else {
                            Some(entry.clone())
                        }
                    })
                    .collect::<Vec<_>>();
                rows
            },
            selected: None,
            idx: None,
            error_text: None,
            show_hidden: false,

            is_exit: false,
        })
    }

    fn rescan_dir(&mut self) -> Result<()> {
        self.rows = {
            let dir = fs::read_dir(&self.current_dir)?;
            let mut rows = dir
                .map(|entry| FileEntry::from_dir_entry(&entry.unwrap()).unwrap())
                .collect::<Vec<_>>();
            if !self.show_hidden {
                rows = rows
                    .iter()
                    .filter_map(|entry| {
                        if entry.is_hidden {
                            None
                        } else {
                            Some(entry.clone())
                        }
                    })
                    .collect::<Vec<_>>();
            }

            if &self.current_dir != Path::new("/") {
                rows.insert(0, up_dir(&self.current_dir));
            };
            rows.sort_by_key(|key| key.file_name.clone());
            rows
        };

        self.rows.sort_by_key(|key| key.file_name.clone());
        if self.rows.len() > 0 {
            self.idx = Some(0);
            self.ts.select(Some(0));
        }
        Ok(())
    }

    fn remove_error_msg(&mut self) {
        self.error_text = None;
    }

    pub fn run(&mut self, term: &mut DefaultTerminal) -> Result<()> {
        while !self.is_exit {
            term.draw(|frame| self.ui(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn set_current_dir(&mut self, dir: PathBuf) {
        self.current_dir = dir;
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key_event(key),
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::F(10) | KeyCode::Char('q') | KeyCode::Char('й') => self.is_exit = true,
            KeyCode::Down | KeyCode::Char('j') => {
                if self.idx.unwrap_or(0) < self.rows.len() - 1 {
                    self.ts.select_next();
                    self.rows
                        .get(self.ts.selected().unwrap_or(0))
                        .cloned()
                        .clone_into(&mut self.selected);
                    self.idx = Some(self.ts.selected().unwrap_or(0));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.ts.select_previous();
                self.rows
                    .get(self.ts.selected().unwrap_or(0))
                    .cloned()
                    .clone_into(&mut self.selected);
                self.idx = Some(self.ts.selected().unwrap_or(0));
            }
            KeyCode::Home | KeyCode::Char('h') => {
                if self.rows.len() > 0 {
                    self.idx = Some(0);
                    self.ts.select(self.idx);
                }
            }
            KeyCode::End | KeyCode::Char('l') => {
                if self.rows.len() > 0 {
                    self.idx = Some(self.rows.len() - 1);
                    self.ts.select(self.idx);
                }
            }
            KeyCode::F(8) => {
                if let Some(selected) = &self.selected {
                    if let Err(why) = selected.remove() {
                        self.error_text = Some(why.to_string());
                    } else {
                        if let Err(why) = self.rescan_dir() {
                            self.error_text = Some(why.to_string());
                        }
                    }
                }
            }
            KeyCode::Enter => {
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

                let rescan = self.rescan_dir();
                if rescan.is_err() {
                    self.error_text = Some(rescan.err().unwrap().to_string());
                }
            }
            KeyCode::Char('.') => {
                self.show_hidden = !self.show_hidden;

                let rescan = self.rescan_dir();
                if rescan.is_err() {
                    self.error_text = Some(rescan.err().unwrap().to_string());
                }
            }
            KeyCode::Esc => self.remove_error_msg(),
            _ => {}
        }
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

        let fcols = self.colors.footer;
        let footer = Line::from(vec![
            "F1".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            " ".into(),
            "Help".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
            "  ".into(),
            "F2".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            " ".into(),
            "Info".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
            "  ".into(),
            "F3".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            " ".into(),
            "View".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
            "  ".into(),
            "F4".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            " ".into(),
            "Edit".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
            "  ".into(),
            "F5".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            " ".into(),
            "Copy".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
            "  ".into(),
            "F6".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            " ".into(),
            "Move".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
            "  ".into(),
            "F7".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            " ".into(),
            "NDir".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
            "  ".into(),
            "F8".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            " ".into(),
            "Delete".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
            "  ".into(),
            "F9".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            " ".into(),
            "Menu".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
            "  ".into(),
            "F10".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            "/".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
            "q".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            "/".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
            "й".set_style(get_style(fcols.key_code, fcols.key_code_modifier)),
            " ".into(),
            "Exit".set_style(get_style(fcols.key_title, fcols.key_title_modifier)),
        ])
        .style(
            Style::default()
                .bg(color_from_u8(fcols.background).unwrap_or_default())
                .fg(color_from_u8(fcols.key_title).unwrap_or_default()),
        );
        frame.render_widget(footer, chunks[2]);

        let fcols = self.colors.panels;
        let mut files_block = Block::bordered()
            .border_set(border::DOUBLE)
            .style(
                Style::new()
                    .bg(color_from_u8(fcols.background).unwrap_or_default())
                    .fg(color_from_u8(fcols.file).unwrap_or_default()),
            )
            .set_style(get_style(fcols.background, fcols.file_modifier))
            .title(
                match &self.selected {
                    Some(selected) => format!(
                        " {} ({}/{}) ",
                        selected.file_name.to_string_lossy(),
                        self.idx.unwrap_or(0) + 1,
                        self.rows.len()
                    ),
                    None => String::new(),
                }
                .bg(Color::Gray)
                .fg(Color::Black),
            )
            .title_top(
                Line::from(format!(" {} files in this dir ", self.rows.len()))
                    .right_aligned()
                    .bg(Color::Gray)
                    .fg(Color::Black),
            )
            .title_top(
                Line::from(format!(
                    " {} ",
                    fs::canonicalize(&self.current_dir).unwrap().display()
                ))
                .centered()
                .bg(Color::Gray)
                .fg(Color::Black),
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
                item.file_name
                    .to_string_lossy()
                    .to_string()
                    .set_style(get_file_color(&item, fcols)),
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
            .row_highlight_style(Style::new().bg(Color::Cyan))
            .block(files_block.clone());

        StatefulWidget::render(
            table,
            files_block.inner(frame.area()),
            frame.buffer_mut(),
            &mut self.ts,
        );
    }
}
