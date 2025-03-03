/*****************************************************************************
 * f - simple and stupid console pseudographic file manager for UNIX systems *
 *                                                                           *
 * Author    : Michail Krasnov <michail383krasnov@mail.ru>                   *
 * Version   : 0.1.0 (see env!("CARGO_PKG_VERSION"))                         *
 * Repository: https://github.com/mskrasnov/f                                *
 * License   : MIT (see '/LICENSE' file in this repository)                  *
 *****************************************************************************/

pub mod consts;
pub mod ftype; // complete
pub mod history;
pub mod init; // complete
pub mod os_release;
pub mod recycle_bin; // complete
pub mod traits; // complete
pub mod tui;
pub mod utils; // complete
pub mod conf;

// NOTE: experimental module
pub mod tui_new;

use anyhow::Result;
use ftype::*;
use std::env;

fn main() -> Result<()> {
    init::create_dirs()?;

    let args = env::args().collect::<Vec<_>>();
    let binding = ".".to_string();
    let fpth = args.get(1).unwrap_or(&binding);

    // Создание экземпляра `tui` до инициализации терминала для того, чтобы
    // гарантировать, что `ratatui` будет проинициализирован только в случае
    // успешного создания экземпляра.
    let mut tui = tui::F::new(fpth)?;

    let mut term = ratatui::init();
    let rslt = tui.run(&mut term);
    ratatui::restore();

    rslt
}
