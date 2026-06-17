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

    /// Lance la boucle UI principale
    pub fn run(&self, map: &Map, mut app_state: UIApp, bus: &CommunicationBus) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // État de l'UI
        let mut ui_state = UIState::new();

        // Boucle principale
        loop {
            // Vérifier les inputs
            app_state.handle_input()?;
            if !app_state.running {
                break;
            }

            // Traitement des messages de communication
            bus.process_messages(&mut ui_state);

            // Rendu
            terminal.draw(|f| self.draw(f, map, &ui_state))?;
        }

        // Cleanup terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    /// Dessine l'UI dans le terminal
    fn draw(&self, f: &mut ratatui::Frame, map: &Map, state: &UIState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    Constraint::Length(3),      // Titre
                    Constraint::Min(12),         // Carte
                    Constraint::Length(3),      // Compteurs
                    Constraint::Length(4),      // Ressources découvertes
                ]
                .as_ref(),
            )
            .split(f.area());

        // Titre
        let title = Paragraph::new("🚀 RESOURCE COLLECTION SIMULATION")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Carte
        let map_widget = self.render_map(map, state);
        f.render_widget(map_widget, chunks[1]);

        // Compteurs
        let counters = self.render_counters(state);
        f.render_widget(counters, chunks[2]);

        // Ressources découvertes
        let resources = self.render_discovered_resources(state);
        f.render_widget(resources, chunks[3]);
    }

    /// Rendu de la carte dans Ratatui
    fn render_map(&self, map: &Map, _state: &UIState) -> Paragraph {
        let mut lines = vec![];

        for y in 0..map.height {
            let mut line_spans = vec![];
            for x in 0..map.width {
                let tile = map.tiles[y][x];

                let (symbol, color) = match tile {
                    Tile::Empty => ("·", Color::DarkGray),
                    Tile::Obstacle => ("█", Color::Red),
                    Tile::Base => ("#", Color::Yellow),
                    Tile::Resource(ResourceKind::Energy) => ("⚡", Color::Green),
                    Tile::Resource(ResourceKind::Crystal) => ("💎", Color::Magenta),
                };

                line_spans.push(Span::styled(symbol, Style::default().fg(color)));
            }
            lines.push(Line::from(line_spans));
        }

        Paragraph::new(lines)
            .block(Block::default().title("Map").borders(Borders::ALL))
            .wrap(Wrap { trim: true })
    }

    /// Affiche les compteurs
    fn render_counters(&self, state: &UIState) -> Paragraph {
        let content = vec![
            Line::from(vec![
                Span::styled("⚡ Énergie: ", Style::default().fg(Color::Green)),
                Span::raw(format!("{}", state.energy_collected)),
            ]),
            Line::from(vec![
                Span::styled("💎 Cristaux: ", Style::default().fg(Color::Magenta)),
                Span::raw(format!("{}", state.crystals_collected)),
            ]),
        ];

        Paragraph::new(content)
            .block(Block::default().title("Ressources").borders(Borders::ALL))
    }

    /// Affiche les ressources découvertes
    fn render_discovered_resources(&self, state: &UIState) -> Paragraph {
        let mut content = vec![];

        if state.discovered_resources.is_empty() {
            content.push(Line::from(Span::raw("Aucune ressource découverte")));
        } else {
            for (pos, resource) in state.discovered_resources.iter().take(3) {
                let kind_str = match resource.kind {
                    ResourceKind::Energy => "⚡ Énergie",
                    ResourceKind::Crystal => "💎 Cristaux",
                };
                content.push(Line::from(format!(
                    "{} à ({},{}): {}",
                    kind_str, pos.x, pos.y, resource.quantity
                )));
            }
        }

        Paragraph::new(content)
            .block(
                Block::default()
                    .title("Découvertes (Q pour quitter)")
                    .borders(Borders::ALL),
            )
    }
}
