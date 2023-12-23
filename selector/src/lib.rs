mod app;
mod tui;
mod ui;
mod update;

use std::{time::Duration, io::Result};
use crossterm::event::poll;
use crate::update::update;
use app::Selector;

pub fn select<'a>(items: Vec<String>, help_msg: Option<&'a str>, err_msg: Option<&'a str>) -> Result<String> {
    let mut app = Selector::new(items, help_msg, err_msg);
    let mut tui = tui::Tui::enter()?;

    while !app.exit {
        tui.draw(&mut app)?;
        if poll(Duration::from_secs(2))? {
            update(&mut app, &mut tui)?;
        }
    }

    tui.exit()?;
    Ok(app.items[app.table_state.selected().unwrap()].to_string())
}
