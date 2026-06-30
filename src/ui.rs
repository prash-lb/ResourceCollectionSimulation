use crate::communication::CommunicationBus;
use resource_collection_simulation::{Map, UIState, Tile, ResourceKind};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::io;

pub struct UIApp {
    pub running: bool,
}

impl UIApp {
    pub fn new() -> Self {
        UIApp { running: true }
    }

    pub fn handle_input(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    self.running = false;
                }
            }
        }
        Ok(())
    }
}

pub struct UIRenderer {
    pub width: u16,
    pub height: u16,
}

impl UIRenderer {
    pub fn new(width: u16, height: u16) -> Self {
        UIRenderer { width, height }
    }

    pub fn run(&self, map: &Map, mut app_state: UIApp, bus: &CommunicationBus) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut ui_state = UIState::new();

        loop {
            app_state.handle_input()?;
            if !app_state.running {
                break;
            }

            bus.drain_messages(&mut ui_state);

            terminal.draw(|f| self.draw(f, map, &ui_state))?;
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn draw(&self, f: &mut ratatui::Frame, map: &Map, state: &UIState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Min(12),
                    Constraint::Length(3),
                    Constraint::Length(4),
                ]
                .as_ref(),
            )
            .split(f.area());

        let title = Paragraph::new("RESOURCE COLLECTION SIMULATION")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        let map_widget = self.render_map(map, state);
        f.render_widget(map_widget, chunks[1]);

        let counters = self.render_counters(state);
        f.render_widget(counters, chunks[2]);

        let resources = self.render_discovered_resources(state);
        f.render_widget(resources, chunks[3]);
    }

    fn render_map(&self, map: &Map, _state: &UIState) -> Paragraph<'_> {
        let mut lines = vec![];

        for y in 0..map.height {
            let mut line_spans = vec![];
            for x in 0..map.width {
                let tile = map.tiles[y][x];

                let (symbol, color) = match tile {
                    Tile::Empty => (".", Color::DarkGray),
                    Tile::Obstacle => ("O", Color::Red),
                    Tile::Base => ("#", Color::Yellow),
                    Tile::Resource(ResourceKind::Energy) => ("E", Color::Green),
                    Tile::Resource(ResourceKind::Crystal) => ("C", Color::Magenta),
                };

                line_spans.push(Span::styled(symbol, Style::default().fg(color)));
            }
            lines.push(Line::from(line_spans));
        }

        Paragraph::new(lines)
            .block(Block::default().title("Map").borders(Borders::ALL))
            .wrap(Wrap { trim: true })
    }

    fn render_counters(&self, state: &UIState) -> Paragraph<'_> {
        let content = vec![
            Line::from(vec![
                Span::styled("Energie: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{}", state.energy_collected)),
                Span::styled("  Cristaux: ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("{}", state.crystals_collected)),
            ]),
        ];

        Paragraph::new(content)
            .block(Block::default().title("Ressources collectees").borders(Borders::ALL))
    }

    fn render_discovered_resources(&self, state: &UIState) -> Paragraph<'_> {
        let mut content = vec![];

        if state.discovered_resources.is_empty() {
            content.push(Line::from(Span::raw("Aucune ressource decouverte")));
        } else {
            for (pos, resource) in state.discovered_resources.iter().take(3) {
                let kind_str = match resource.kind {
                    ResourceKind::Energy => "Energie",
                    ResourceKind::Crystal => "Cristaux",
                };
                content.push(Line::from(format!(
                    "{} a ({},{}): {}",
                    kind_str, pos.x, pos.y, resource.quantity
                )));
            }
        }

        Paragraph::new(content)
            .block(
                Block::default()
                    .title("Decouvertes (Q pour quitter)")
                    .borders(Borders::ALL),
            )
    }
}
