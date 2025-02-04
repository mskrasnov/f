//! Some utilities and helpers

use std::env::var;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;

use crate::ftype::{FileEntry, FileType};

/// Get path to the user home directory
pub fn get_home() -> PathBuf {
    Path::new(&var("HOME").unwrap_or("/tmp".to_string())).to_path_buf()
}

/// Get path to the parent directory
pub fn parent_dir<P: AsRef<Path>>(current: P) -> Result<FileEntry> {
    let current_canon = fs::canonicalize(&current)?;

    Ok(FileEntry {
        file_name: OsString::from_str(".. [UP]").unwrap(),
        path: current_canon.clone().parent().unwrap_or(Path::new("/")).to_path_buf(),
        byte_size: current_canon.metadata()?.len(),
        is_hidden: false,
        file_type: FileType::ParentDirectory,
    })
}

/// Read directory contents
pub fn read_dir<P: AsRef<Path>>(pth: P, show_hidden: bool) -> Result<Vec<FileEntry>> {
    let dir = fs::read_dir(&pth)?;
    let mut rows = dir
        // Используем только то, что можем прочитать
        .filter_map(|entry| entry.ok())
        // Используем только то, что можем обернуть в FileEntry
        .filter_map(|entry| FileEntry::from_dir_entry(&entry).ok())
        // Обрабатываем возможность отключения отображения скрытых файлов
        .filter_map(|entry| {
            if entry.is_hidden && !show_hidden {
                None
            } else {
                Some(entry.clone())
            }
        })
        // Собираем красивый вектор из этой поебени
        .collect::<Vec<_>>();

    /* Далее требуется выполнить сортировку вектора, потому что в исходном Vec файлы
     * хранятся вразнобой. После сортировки добавляем в вектор путь, указывающий на
     * родительский каталог. Если добавить путь к нему ДО сортировки, то этот каталог
     * может оказаться не на первом месте, когда как он должен всегда находиться
     * на первом месте.
     */
    rows.sort_by_key(|key| key.file_name.clone());

    if pth.as_ref() != Path::new("/") {
        rows.insert(0, parent_dir(&pth)?);
        // rows.insert(1, FileEntry {
        //     file_name: OsString::from_str("..").unwrap(),
        //     path: Path::new("/").to_path_buf(),
        //     byte_size: 0,
        //     file_type: FileType::Directory,
        //     is_hidden: false,
        // });
    }

    // rows = rows
    //     .iter()
    //     .filter_map(|entry| {
    //         if entry.is_hidden {
    //             None
    //         } else {
    //             Some(entry.clone())
    //         }
    //     })
    //     .collect::<Vec<_>>();

    Ok(rows)
}
