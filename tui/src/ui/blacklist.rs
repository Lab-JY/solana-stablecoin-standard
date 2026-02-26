use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    if !app.dashboard.compliance_enabled {
        let text = vec![
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "  Compliance module is not enabled (SSS-1 mode)",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Blacklist management requires SSS-2 preset.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(Span::styled(
                "  Initialize with --preset sss-2 to enable compliance features.",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let block = Paragraph::new(text).block(
            Block::default()
                .title(" Blacklist Management (SSS-2) ")
                .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        f.render_widget(block, area);
        return;
    }

    if app.blacklist_entries.is_empty() {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No blacklisted addresses.",
                Style::default().fg(Color::Green),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Compliance module is active. Use the CLI to manage blacklists:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "    sss-token blacklist add <address> --reason \"OFAC sanctions\"",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "    sss-token blacklist remove <address>",
                Style::default().fg(Color::Yellow),
            )),
        ];

        let block = Paragraph::new(text).block(
            Block::default()
                .title(" Blacklist Management (SSS-2) ")
                .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        );

        f.render_widget(block, area);
        return;
    }

    let items: Vec<ListItem> = app
        .blacklist_entries
        .iter()
        .enumerate()
        .map(|(i, addr)| {
            let line = Line::from(vec![
                Span::styled(
                    format!(" {:>3}. ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    " BLOCKED ",
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(addr, Style::default().fg(Color::White)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(format!(
                " Blacklist ({} addresses) ",
                app.blacklist_entries.len()
            ))
            .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)),
    );

    f.render_widget(list, area);
}
