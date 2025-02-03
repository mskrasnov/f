//! Widget for show current dir contents

use anyhow::Result;
use std::{fs, path::Path};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{List, ListItem, ListState, StatefulWidget},
};

use crate::FileEntry;

pub struct CurrentDir {
    /// Directory entries
    items: Vec<FileEntry>,

    state: ListState,
}

impl CurrentDir {
    pub fn new<P: AsRef<Path>>(pth: P) -> Result<Self> {
        let contents = fs::read_dir(&pth)?
            .map(|entry| entry.unwrap())
            .collect::<Vec<_>>();
        let mut items = Vec::with_capacity(contents.len());

        for entry in contents {
            items.push(FileEntry::from_dir_entry(&entry)?);
        }

        Ok(Self {
            state: ListState::default(),
            items,
        })
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

impl StatefulWidget for &CurrentDir {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State)
    where
        Self: Sized,
    {
        // let title = Line::from(vec![
        //     " ".into(),
        //     format!("{}", &self.path.display()).yellow().bold(),
        //     " (files: ".dim(),
        //     self.count.to_string().dim(),
        //     ") ".dim(),
        // ]);

        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| ListItem::new(i.file_name.to_str().unwrap_or("UNKNOWN FILE")))
            .collect();
        let list = List::new(items);

        // let block = Block::bordered()
        // .title(title)
        // .border_set(border::THICK);

        // list.render(area, buf);
        StatefulWidget::render(list, area, buf, state);
    }
}
