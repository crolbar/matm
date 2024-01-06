use crossterm::event::{KeyCode, Event, self, MouseEventKind, KeyModifiers, MouseEvent};
use std::io::Result;

use crate::{app::Selector, tui::Tui};

pub fn update(app: &mut Selector, tui: &mut Tui) -> Result<()> {
    if let Ok(event) = event::read() {
        if let Event::Key(key) = event {

            if 
                key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c') ||
                key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('z') ||
                key.modifiers == KeyModifiers::ALT && key.code == KeyCode::Char('q') ||
                key.code == KeyCode::Esc
            {
                tui.exit()?;
                std::process::exit(0)
            }

            // ctrl + backspace
            if 
                key.modifiers == KeyModifiers::CONTROL &&
                key.code == KeyCode::Char('h')
            {
                app.search.needle.clear();

                if !app.search.haystack.is_empty() {
                    app.search.revert_items(&mut app.items)
                }
            }

            else if 
                key.modifiers == KeyModifiers::ALT | KeyModifiers::SHIFT  ||
                key.modifiers == KeyModifiers::ALT
            {
                if let KeyCode::Char(char) = key.code {
                    match char {
                        'j' => app.sel_next_item(),
                        'k' => app.sel_prev_item(),
                        'g' => app.table_state.select(Some(0)),
                        'G' => app.table_state.select(Some(app.items.len().saturating_sub(1))),
                        _ => ()
                    }
                }
            } else {
                match key.code {
                    KeyCode::Down => app.sel_next_item(),
                    KeyCode::Up => app.sel_prev_item(),
                    KeyCode::Enter => { 
                        app.exit = !app.items.is_empty()
                    }

                    KeyCode::PageUp | KeyCode::Home => app.table_state.select(Some(0)),
                    KeyCode::PageDown | KeyCode::End => app.table_state.select(Some(app.items.len().saturating_sub(1))),

                    KeyCode::Backspace => {
                        app.search.pop_char();
                        app.search.search_trough_origin_items(&mut app.items);

                        if app.items.is_empty() {
                            app.search.revert_items(&mut app.items)
                        }
                    },

                    KeyCode::Char(char) => {
                        app.search.push_char(char);
                        app.search.search_trough_origin_items(&mut app.items);

                        if app.table_state.selected().unwrap() >= app.items.len() {
                            app.table_state.select(Some(0))
                        }
                    }

                    _ => ()
                }
            }
        } else

        if let Event::Mouse(mouse_ev) = event {
            match mouse_ev.kind {
                MouseEventKind::ScrollDown => app.sel_next_item(),
                MouseEventKind::ScrollUp => app.sel_prev_item(),
                MouseEventKind::Down(_) => {
                    app.handle_mb_down(mouse_ev);
                    tui.draw(app)?;

                    if let Ok(Event::Mouse(MouseEvent { kind: MouseEventKind::Up(_), .. })) = event::read() {
                        if crossterm::event::poll(std::time::Duration::from_millis(200))? {
                            if let Ok(Event::Mouse(MouseEvent { kind: MouseEventKind::Down(_), .. })) = event::read() {
                                app.exit = true
                            }
                        }
                    }
                },
                _ => ()
            }
        }
    }
    Ok(())
}
