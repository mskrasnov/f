/*****************************************************************************
 * f - simple and stupid console pseudographic file manager for UNIX systems *
 *                                                                           *
 * Author    : Michail Krasnov <michail383krasnov@mail.ru>                   *
 * Version   : 0.1.0 (see env!("CARGO_PKG_VERSION"))                         *
 * Repository: https://github.com/mskrasnov/f                                *
 * License   : MIT (see '/LICENSE' file in this repository)                  *
 *****************************************************************************/

pub mod consts;
pub mod ftype;
pub mod history;
pub mod traits;
pub mod tui;
pub mod utils;
pub mod os_release;
pub mod recycle_bin;

// NOTE: experimental module
pub mod tui_new;

use anyhow::Result;
use ftype::*;
use std::env;

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let binding = ".".to_string();
    let fpth = args.get(1).unwrap_or(&binding);

    let mut term = ratatui::init();
    let rslt = tui_new::F::new(fpth)?.run(&mut term);
    ratatui::restore();

    rslt
}
