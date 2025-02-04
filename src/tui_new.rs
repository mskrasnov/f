//! Experimental TUI for `f`

use crate::consts::{PROG_NAME, PROG_VER};
use crate::tui::colors::{color_from_u8, get_style, Colors, Modifier, Panels};
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

/// Get style for list_files panel
///
/// ## Usage
///
/// ```
/// let entry = FileEntry { ... };
/// let fcol = FileColor {
///     entry: &entry,
///     cols: Panels::default(),
///     selected: true,
/// };
/// let color = fcol.style();
/// ```
#[derive(Clone)]
struct FileColor<'a> {
    entry: &'a FileEntry,
    cols: Panels,
    selected: bool,
}

impl<'a> FileColor<'a> {
    fn get_bg(&self) -> Option<Color> {
        color_from_u8(self.cols.selection_color)
    }

    fn get_fg_hidden_u8(&self, c: u8) -> u8 {
        if c <= 38 {
            c + 60
        } else {
            c - 60
        }
    }

    fn get_fg_unselected(&self) -> Option<Color> {
        if self.entry.is_hidden {
            match self.entry.file_type {
                FileType::Directory | FileType::ParentDirectory => color_from_u8(self.cols.dir),
                FileType::Link => color_from_u8(self.cols.link),
                FileType::Special => color_from_u8(self.cols.special_file),
                _ => color_from_u8(self.cols.file),
            }
        } else {
            /* Для скрытого файла нужно использовать в зависимости от основного цвета:
             *  - более светлый цвет. Например, если для обычного файла был Blue, для
             *    скрытого - Light Blue;
             *  - более тёмный цвет. Например, Light Blue -> Blue;
             *
             * Для этого смотрим значение основного цвета и прибавляем/вычитаем из
             * него 60, поскольку разница между обычным и светлым вариантом - 60.
             */
            color_from_u8(match self.entry.file_type {
                FileType::Directory | FileType::ParentDirectory => {
                    self.get_fg_hidden_u8(self.cols.dir)
                }
                FileType::Link => self.get_fg_hidden_u8(self.cols.link),
                FileType::Special => self.get_fg_hidden_u8(self.cols.special_file),
                _ => self.get_fg_hidden_u8(self.cols.file),
            })
        }
    }

    fn get_unselected_u8(&self, c1: Option<u8>, c2: u8) -> u8 {
        match c1 {
            Some(color_unselected) => color_unselected,
            None => c2,
        }
    }

    fn get_fg_selected(&self) -> Option<Color> {
        /* Для упрощения кода для скрытых файлов будет использоваться тот же цвет
         * выделения, что и для не-скрытого. */
        match self.entry.file_type {
            FileType::Directory | FileType::ParentDirectory => {
                color_from_u8(self.get_unselected_u8(self.cols.dir_selected, self.cols.dir))
            }
            FileType::Link => {
                color_from_u8(self.get_unselected_u8(self.cols.link_selected, self.cols.link))
            }
            FileType::Special => color_from_u8(
                self.get_unselected_u8(self.cols.special_file_selected, self.cols.special_file),
            ),
            _ => color_from_u8(self.get_unselected_u8(self.cols.file_selected, self.cols.file)),
        }
    }

    fn get_modifier(&self) -> Modifier {
        Modifier::from(match self.entry.file_type {
            FileType::File | FileType::FileExecutable => self.cols.file_modifier.unwrap_or(8),
            FileType::Directory | FileType::ParentDirectory => self.cols.dir_modifier.unwrap_or(8),
            FileType::Link => self.cols.link_modifier.unwrap_or(8),
            FileType::Special => self.cols.special_file_modifier.unwrap_or(8),
        })
    }

    pub fn style(&self) -> Style {
        let mut style = Style::default()
            .bg(if self.selected {
                self.get_bg()
                    .unwrap_or(color_from_u8(self.cols.background).unwrap_or_default())
            } else {
                color_from_u8(self.cols.background).unwrap_or_default()
            })
            .fg(if self.selected {
                self.get_fg_selected().unwrap()
                // self.get_bg().unwrap()
            } else {
                self.get_fg_unselected().unwrap_or_default()
            });

        let modifier = self.get_modifier();
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
}

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
                    if let Err(why) = selected.remove() {
                        self.error_text = Some(why.to_string());
                    } else {
                        if let Err(why) = self.rescan_dir() {
                            self.error_text = Some(why.to_string());
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

        let fcols = self.colors.panels;
        let mut files_block = Block::bordered()
            .border_set(border::DOUBLE)
            .style(
                Style::default()
                    .bg(color_from_u8(fcols.background).unwrap_or_default())
                    .fg(color_from_u8(fcols.file).unwrap_or_default()),
            )
            .set_style(get_style(fcols.background, fcols.file_modifier))
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
                Line::from(format!(" {} files in this dir ", self.rows.len()))
                    .right_aligned()
                    // .bg(Color::Gray)
                    // .fg(Color::Black),
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
            .row_highlight_style(Style::new().bg(Color::Cyan))
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
