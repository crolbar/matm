use ratatui::{prelude::*, widgets::*};
use crate::app::Selector;

pub fn render(app: &mut Selector, frame: &mut Frame) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(frame.size());

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(13),
            Constraint::Percentage(2),
            Constraint::Percentage(70),
            Constraint::Percentage(2),
            Constraint::Percentage(13),
        ])
        .split(layout[1]);

    let help_rect = layout[1];
    let table_rect = layout[2];
    let err_rect = layout[3];
    app.set_table_rect(table_rect);
    

    {
        let rows = app.items.iter().map(|i| {
            let cells = [Cell::from(*i)];
            Row::new(cells)
        });
        let dark_red_col = Color::Rgb(100, 0, 0);

        frame.render_stateful_widget(
            Table::new(rows, [Constraint::Percentage(100)])
            .block(
                Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(dark_red_col))
            )
            .highlight_style(
                Style::default().bg(dark_red_col)
            ).highlight_symbol("> "),
            table_rect,
            &mut app.table_state
        )
    }

    {
        if let Some(help_msg) = app.help_msg {
            frame.render_widget(
                Paragraph::new(help_msg)
                .alignment(Alignment::Center),
                help_rect
            );
        }

        if let Some(err_msg) = app.err_msg {
            frame.render_widget(
                Paragraph::new(err_msg).red()
                .alignment(Alignment::Center),
                err_rect
            );
        }
    }
}
