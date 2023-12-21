use ratatui::{widgets::TableState, prelude::Rect};
use crossterm::event::MouseEvent;

#[derive(Default)]
pub struct Selector<'a> {
    pub items: Vec<&'a str>,
    pub table_state: TableState,
    pub table_rect: Rect,
    pub exit: bool,
    pub help_msg: Option<&'a str>,
    pub err_msg: Option<&'a str>,
}

impl<'a> Selector<'a> {
    pub fn new(items: Vec<&'a str>, msg: Vec<&'a str>) -> Self {
        Selector { 
            items,
            help_msg: msg.get(0).copied(),
            err_msg: msg.get(1).copied(),
            table_state: TableState::default().with_selected(Some(0)),
            ..Default::default()
        }
    }

    pub fn sel_next_item(&mut self) {
        self.table_state.select(
            Some( (self.table_state.selected().unwrap() + 1) % self.items.len() )
        )
    }
    pub fn sel_prev_item(&mut self) {
        let sel = self.table_state.selected().unwrap();
        self.table_state.select(
            Some(
                match sel {
                    0 => self.items.len() - 1,
                    _ => sel - 1
                }
            )
        )
    }

    pub fn set_table_rect(&mut self, rect: Rect) {
        self.table_rect = rect
    }

    pub fn handle_mb_down(&mut self, click_ev: MouseEvent) {
        if 
            click_ev.column >= self.table_rect.x &&
            click_ev.column <= self.table_rect.x + self.table_rect.width &&
            click_ev.row >= self.table_rect.y && 
            click_ev.row <= self.table_rect.y + self.table_rect.height
        {
            let click_item_idex = (click_ev.row - self.table_rect.y) as usize;

            if click_item_idex == self.table_state.selected().unwrap() {
                self.exit = true
            } 

            if click_ev.row > self.items.len() as u16 + self.table_rect.y {
                self.table_state.select(Some(self.items.len() - 1))
            } else {
                self.table_state.select(Some(click_item_idex))
            }
        }
    }
}
