mod app;
mod tui;
mod ui;
mod update;

use std::{time::Duration, io::Result};
use crossterm::event::poll;
use crate::update::update;
use app::Selector;

/// First item of the secord vec is the help message an the secord is err message
pub fn select<'a>(items: Vec<&'a str>, msg: Vec<&'a str>) -> Result<&'a str> {
    let mut app = Selector::new(items, msg);
    let mut tui = tui::Tui::enter()?;

    while !app.exit {
        tui.draw(&mut app)?;
        if poll(Duration::from_secs(2))? {
            update(&mut app, &mut tui)?;
        }
    }

    tui.exit()?;
    Ok(app.items[app.table_state.selected().unwrap()])
}
