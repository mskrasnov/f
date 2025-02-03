//! Text user interface for `f`

use crate::consts::{PROG_NAME, PROG_VER};
use crate::tui::colors::{color_from_u8, get_style, Colors};
use crate::{traits::Toml, FileEntry, FileType};

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

pub struct F {
    colors: Colors,
    ts: TableState,
    current_dir: PathBuf,
    rows: Vec<FileEntry>,
    selected: Option<FileEntry>,

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
                rows.insert(
                    0,
                    FileEntry {
                        file_name: OsString::from_str("..").unwrap(),
                        path: Path::new("..").to_path_buf(),
                        byte_size: 4096,
                        file_type: FileType::Directory,
                    },
                );
                rows.sort_by_key(|key| key.file_name.clone());
                rows
            },
            selected: None,
            is_exit: false,
        })
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
                self.ts.select_next();
                self.rows
                    .get(self.ts.selected().unwrap_or(0))
                    .cloned()
                    .clone_into(&mut self.selected);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.ts.select_previous();
                self.rows
                    .get(self.ts.selected().unwrap_or(0))
                    .cloned()
                    .clone_into(&mut self.selected);
            }
            KeyCode::F(8) => {
                if let Some(selected) = &self.selected {
                    selected.remove();
                }
            }
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
        let title = Line::from(format!("The {} file manager ver.{}", PROG_NAME, PROG_VER))
            .set_style(get_style(tcols.background, tcols.text_modifier))
            .fg(color_from_u8(tcols.text).unwrap_or_default())
            .bg(color_from_u8(tcols.background).unwrap_or_default())
            .centered();
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
        let files_block = Block::bordered()
            .border_set(border::DOUBLE)
            .style(
                Style::new()
                    .bg(color_from_u8(fcols.background).unwrap_or_default())
                    .fg(color_from_u8(fcols.file).unwrap_or_default()),
            )
            .set_style(get_style(fcols.background, fcols.file_modifier))
            .title(match &self.selected {
                Some(selected) => format!(" {} ", selected.file_name.to_string_lossy()),
                None => String::new(),
            })
            .title_top(Line::from(format!(" {} files in this dir ", self.rows.len())).right_aligned())
            .title_top(Line::from(format!(" {} ", &self.current_dir.display())).centered());

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
                    match item.file_type {
                        FileType::Directory => color_from_u8(fcols.dir),
                        FileType::Link => color_from_u8(fcols.link),
                        FileType::Special => color_from_u8(fcols.special_file),
                        _ => color_from_u8(fcols.file),
                    }
                    .unwrap_or_default(),
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
