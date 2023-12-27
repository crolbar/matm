use ratatui::{widgets::TableState, prelude::Rect};
use crossterm::event::MouseEvent;
use rust_fuzzy_search::fuzzy_search;

#[derive(Default)]
pub struct Selector<'a> {
    pub items: Vec<String>,
    pub table_state: TableState,
    pub table_rect: Rect,
    pub exit: bool,
    pub help_msg: Option<&'a str>,
    pub err_msg: Option<&'a str>,
    pub search: Search,
}

#[derive(Default)]
pub struct Search {
    pub haystack: Vec<String>,
    pub needle: String,
}

impl Search {
    pub fn push_char(&mut self, c: char) {
        self.needle.push(c)
    }
    pub fn pop_char(&mut self) {
        if !self.needle.is_empty() {
            self.needle.pop().unwrap();
        }
    }

    fn set_origin_items(&mut self, items: &Vec<String>) {
        self.haystack = items.clone();
    }

    pub fn revert_items(&mut self, items: &mut Vec<String>) {
        *items = self.haystack.clone();
        self.haystack.clear();
    }

    pub fn search_trough_origin_items(&mut self, items: &mut Vec<String>) {
        if self.haystack.is_empty() {
            self.set_origin_items(items);
        }
        items.clear();

        let haystack: Vec<String> = self.haystack
            .iter()
            .map(|i| {
                i.to_lowercase()
                .replace(" ", "")
            }).collect();
        let haystack = haystack.iter().map(|i| i.as_str()).collect::<Vec<&str>>();

        fuzzy_search(
            &self.needle, 
            &haystack.as_slice()
        ).iter()
        .enumerate()
        .for_each(|(i, (_, score))| {
            if *score > 0.2 {
                items.push(self.haystack[i].clone())
            }
        });
    }
}

impl<'a> Selector<'a> {
    pub fn new(items: Vec<String>, help_msg: Option<&'a str>, err_msg: Option<&'a str>) -> Self {
        Selector { 
            items,
            help_msg,
            err_msg,
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
