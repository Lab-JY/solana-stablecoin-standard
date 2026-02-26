use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};

use crate::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),  // Token info
            Constraint::Length(9),  // Supply stats
            Constraint::Min(0),    // Status & details
        ])
        .split(area);

    // Token info block
    draw_token_info(f, app, chunks[0]);

    // Supply stats
    draw_supply_stats(f, app, chunks[1]);

    // Status details
    draw_status_details(f, app, chunks[2]);
}

pub fn draw_mini(f: &mut Frame, app: &App, area: Rect) {
    let decimals = 10u64.pow(app.dashboard.decimals as u32);
    let supply_display = if decimals > 0 {
        app.dashboard.total_supply as f64 / decimals as f64
    } else {
        app.dashboard.total_supply as f64
    };

    let pause_indicator = if app.dashboard.paused {
        Span::styled(" PAUSED ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" ACTIVE ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    };

    let compliance_indicator = if app.dashboard.compliance_enabled {
        Span::styled("SSS-2", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
    } else {
        Span::styled("SSS-1", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
    };

    let text = vec![
        Line::from(vec![
            Span::styled(&app.dashboard.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(" ("),
            Span::styled(&app.dashboard.symbol, Style::default().fg(Color::Yellow)),
            Span::raw(") "),
            compliance_indicator,
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Supply: "),
            Span::styled(
                format!("{:.2}", supply_display),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("Status: "),
            pause_indicator,
        ]),
        Line::from(vec![
            Span::raw("Minters: "),
            Span::styled(
                format!("{}", app.dashboard.active_minters),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let block = Paragraph::new(text).block(
        Block::default()
            .title(" Overview ")
            .title_style(Style::default().fg(Color::Cyan))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(block, area);
}

fn draw_token_info(f: &mut Frame, app: &App, area: Rect) {
    let compliance_tag = if app.dashboard.compliance_enabled {
        Span::styled(" [SSS-2 Compliant] ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
    } else {
        Span::styled(" [SSS-1 Minimal] ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
    };

    let text = vec![
        Line::from(vec![
            Span::styled("Name:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.dashboard.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            compliance_tag,
        ]),
        Line::from(vec![
            Span::styled("Symbol:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.dashboard.symbol, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Decimals:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.dashboard.decimals), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Authority: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.dashboard.authority, Style::default().fg(Color::Cyan)),
        ]),
    ];

    let block = Paragraph::new(text).block(
        Block::default()
            .title(" Token Info ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(block, area);
}

fn draw_supply_stats(f: &mut Frame, app: &App, area: Rect) {
    let decimals = 10u64.pow(app.dashboard.decimals as u32);
    let format_amount = |amount: u64| -> String {
        if decimals > 0 {
            format!("{:.2}", amount as f64 / decimals as f64)
        } else {
            format!("{}", amount)
        }
    };

    let rows = vec![
        Row::new(vec![
            "Total Supply",
            &format_amount(app.dashboard.total_supply),
        ])
        .style(Style::default().fg(Color::Green)),
        Row::new(vec![
            "Total Minted",
            &format_amount(app.dashboard.total_minted),
        ])
        .style(Style::default().fg(Color::Cyan)),
        Row::new(vec![
            "Total Burned",
            &format_amount(app.dashboard.total_burned),
        ])
        .style(Style::default().fg(Color::Red)),
        Row::new(vec![
            "Mint Count",
            &format!("{}", app.dashboard.total_minted),
        ])
        .style(Style::default().fg(Color::DarkGray)),
        Row::new(vec![
            "Burn Count",
            &format!("{}", app.dashboard.total_burned),
        ])
        .style(Style::default().fg(Color::DarkGray)),
    ];

    let widths = [Constraint::Percentage(50), Constraint::Percentage(50)];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title(" Supply Statistics ")
                .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .header(
            Row::new(vec!["Metric", "Value"])
                .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        );

    f.render_widget(table, area);
}

fn draw_status_details(f: &mut Frame, app: &App, area: Rect) {
    let pause_indicator = if app.dashboard.paused {
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::DarkGray)),
            Span::styled("PAUSED", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled("  -- All operations suspended", Style::default().fg(Color::Red)),
        ])
    } else {
        Line::from(vec![
            Span::styled("  Status: ", Style::default().fg(Color::DarkGray)),
            Span::styled("ACTIVE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("  -- Operations running normally", Style::default().fg(Color::Green)),
        ])
    };

    let compliance_line = if app.dashboard.compliance_enabled {
        Line::from(vec![
            Span::styled("  Compliance: ", Style::default().fg(Color::DarkGray)),
            Span::styled("ENABLED", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("  (permanent delegate + transfer hook)", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled("  Compliance: ", Style::default().fg(Color::DarkGray)),
            Span::styled("DISABLED", Style::default().fg(Color::DarkGray)),
            Span::styled("  (SSS-1 mode)", Style::default().fg(Color::DarkGray)),
        ])
    };

    let text = vec![
        Line::from(""),
        pause_indicator,
        Line::from(""),
        compliance_line,
        Line::from(""),
        Line::from(vec![
            Span::styled("  Active Minters: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", app.dashboard.active_minters),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Blacklisted: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", app.blacklist_entries.len()),
                Style::default().fg(if app.blacklist_entries.is_empty() { Color::Green } else { Color::Yellow }),
            ),
        ]),
    ];

    let block = Paragraph::new(text).block(
        Block::default()
            .title(" Status ")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(block, area);
}
