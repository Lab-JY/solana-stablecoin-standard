use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    if app.minters.is_empty() {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No minters configured.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Add minters using the CLI:",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "    sss-token minters add <address> --quota 1000000",
                Style::default().fg(Color::Yellow),
            )),
        ];

        let block = Paragraph::new(text).block(
            Block::default()
                .title(" Minters ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

        f.render_widget(block, area);
        return;
    }

    let rows: Vec<Row> = app
        .minters
        .iter()
        .map(|m| {
            let usage_pct = if m.quota > 0 {
                (m.minted as f64 / m.quota as f64 * 100.0) as u64
            } else {
                0
            };

            let usage_color = if usage_pct >= 90 {
                Color::Red
            } else if usage_pct >= 70 {
                Color::Yellow
            } else {
                Color::Green
            };

            // Create a simple usage bar
            let bar_width = 10;
            let filled = (usage_pct as usize * bar_width) / 100;
            let bar = format!(
                "[{}{}] {}%",
                "=".repeat(filled),
                " ".repeat(bar_width - filled),
                usage_pct
            );

            Row::new(vec![
                m.address.clone(),
                format!("{}", m.quota),
                format!("{}", m.minted),
                format!("{}", m.remaining),
                bar,
            ])
            .style(Style::default().fg(usage_color))
        })
        .collect();

    let widths = [
        Constraint::Min(14),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(18),
    ];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title(format!(" Minters ({}) ", app.minters.len()))
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .header(
            Row::new(vec!["Address", "Quota", "Minted", "Remaining", "Usage"])
                .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        );

    f.render_widget(table, area);
}

pub fn draw_mini(f: &mut Frame, app: &App, area: Rect) {
    if app.minters.is_empty() {
        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  No minters configured",
                Style::default().fg(Color::DarkGray),
            )),
        ];

        let block = Paragraph::new(text).block(
            Block::default()
                .title(" Minters ")
                .title_style(Style::default().fg(Color::Cyan))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

        f.render_widget(block, area);
        return;
    }

    let rows: Vec<Row> = app
        .minters
        .iter()
        .take(5)
        .map(|m| {
            let usage_pct = if m.quota > 0 {
                (m.minted as f64 / m.quota as f64 * 100.0) as u64
            } else {
                0
            };

            let color = if usage_pct >= 90 {
                Color::Red
            } else if usage_pct >= 70 {
                Color::Yellow
            } else {
                Color::Green
            };

            Row::new(vec![
                m.address.clone(),
                format!("{}%", usage_pct),
            ])
            .style(Style::default().fg(color))
        })
        .collect();

    let widths = [Constraint::Min(14), Constraint::Length(8)];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title(format!(" Minters ({}) ", app.minters.len()))
                .title_style(Style::default().fg(Color::Cyan))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .header(
            Row::new(vec!["Address", "Usage"])
                .style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(table, area);
}
