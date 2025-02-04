//! Constants and global variables

/**********************************************************
 *                Information about program               *
 **********************************************************/
pub const PROG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PROG_VER: &str = env!("CARGO_PKG_VERSION");
pub const PROG_EDITION: &str = "I";
pub const PROG_COPYRIGHT: &str = "Copyright (C) 2025 Michail Krasnov\n\
                                      <michail383krasnov@mail.ru>";
pub const PROG_REPOSITORY: &str = "https://github.com/mskrasnov/f";

/**********************************************************
 *                       Some pathes                      *
 **********************************************************/
pub const CONF_DIR: &str = ".config/f/";
pub const MASTER_CONF: &str = ".config/f/master.conf";
pub const HISTORY_FILE: &str = ".cofnig/f/history";
pub const UNAME_FILE: &str = "/proc/version";
pub const OS_RELEASE_FILE: &str = "/etc/os-release";
pub const RECYCLE_BIN_DIR: &str = ".local/share/f_bin/";
pub const RECYCLE_BIN_META: &str = ".local/share/f_bin/f_bin.toml";
