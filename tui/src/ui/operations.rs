use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    if app.operations.is_empty() {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No operations recorded yet.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Operations will appear here as they occur on-chain.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Mint, burn, freeze, thaw, pause, blacklist events",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  are all tracked and displayed in real-time.",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let block = Paragraph::new(text).block(
            Block::default()
                .title(" Recent Operations ")
                .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        );

        f.render_widget(block, area);
        return;
    }

    let items: Vec<ListItem> = app
        .operations
        .iter()
        .enumerate()
        .map(|(i, op)| {
            let action_color = match op.action.as_str() {
                "Mint" => Color::Green,
                "Burn" => Color::Red,
                "Freeze" => Color::Blue,
                "Thaw" => Color::Cyan,
                "Pause" => Color::Red,
                "Unpause" => Color::Green,
                "Blacklist" => Color::Magenta,
                "Seize" => Color::Red,
                _ => Color::White,
            };

            let line = Line::from(vec![
                Span::styled(
                    format!(" {:>3}. ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("[{}] ", &op.timestamp),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<10}", &op.action),
                    Style::default().fg(action_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(&op.details, Style::default().fg(Color::White)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Recent Operations ({}) ", app.operations.len()))
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(list, area);
}

pub fn draw_mini(f: &mut Frame, app: &App, area: Rect) {
    if app.operations.is_empty() {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No recent operations",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let block = Paragraph::new(text).block(
            Block::default()
                .title(" Operations ")
                .title_style(Style::default().fg(Color::Yellow))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        f.render_widget(block, area);
        return;
    }

    // Show last 5 operations
    let recent: Vec<ListItem> = app
        .operations
        .iter()
        .rev()
        .take(5)
        .map(|op| {
            let action_color = match op.action.as_str() {
                "Mint" => Color::Green,
                "Burn" => Color::Red,
                "Freeze" | "Pause" => Color::Blue,
                _ => Color::White,
            };

            let line = Line::from(vec![
                Span::styled(
                    format!("{:<8}", &op.action),
                    Style::default().fg(action_color),
                ),
                Span::styled(
                    truncate_str(&op.details, 25),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(recent).block(
        Block::default()
            .title(" Recent Ops ")
            .title_style(Style::default().fg(Color::Yellow))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(list, area);
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}
