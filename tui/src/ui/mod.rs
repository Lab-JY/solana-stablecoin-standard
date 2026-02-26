pub mod dashboard;
pub mod operations;
pub mod blacklist;
pub mod minters;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};

use crate::app::{App, Panel};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(0),   // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(f.area());

    // Tab bar
    draw_tabs(f, app, chunks[0]);

    // Main content area - split into two columns
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(chunks[1]);

    // Left column: active panel
    match app.active_panel {
        Panel::Dashboard => dashboard::draw(f, app, main_chunks[0]),
        Panel::Operations => operations::draw(f, app, main_chunks[0]),
        Panel::Blacklist => blacklist::draw(f, app, main_chunks[0]),
        Panel::Minters => minters::draw(f, app, main_chunks[0]),
    }

    // Right column: always show mini dashboard + minters
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_chunks[1]);

    if app.active_panel != Panel::Dashboard {
        dashboard::draw_mini(f, app, right_chunks[0]);
    } else {
        operations::draw_mini(f, app, right_chunks[0]);
    }

    if app.active_panel != Panel::Minters {
        minters::draw_mini(f, app, right_chunks[1]);
    } else {
        operations::draw_mini(f, app, right_chunks[1]);
    }

    // Status bar
    draw_status_bar(f, app, chunks[2]);
}

fn draw_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["1:Dashboard", "2:Operations", "3:Blacklist", "4:Minters"];
    let selected = match app.active_panel {
        Panel::Dashboard => 0,
        Panel::Operations => 1,
        Panel::Blacklist => 2,
        Panel::Minters => 3,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" SSS Admin TUI ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        )
        .select(selected)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");

    f.render_widget(tabs, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status = if let Some(ref err) = app.error_message {
        format!(" ERR: {} | ", err)
    } else {
        String::new()
    };

    let connection_status = if app.dashboard.connected {
        "CONNECTED"
    } else {
        "DISCONNECTED"
    };

    let connection_color = if app.dashboard.connected {
        Color::Green
    } else {
        Color::Red
    };

    let text = Line::from(vec![
        Span::styled(
            format!(" [{}] ", connection_status),
            Style::default().fg(connection_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            status,
            Style::default().fg(Color::Red),
        ),
        Span::styled(
            format!("Mint: {} ", app.dashboard.mint),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            " | Tab: switch panels | r: refresh | q: quit ",
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            format!(" Last: {} ", app.dashboard.last_refresh),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let bar = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(bar, area);
}
