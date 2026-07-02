use std::collections::HashMap;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use resource_collection_simulation::{BaseState, Map, ResourceKind, RobotKind, RobotState, Tile};

pub struct UIRenderer;

impl UIRenderer {
    pub fn new() -> Self {
        UIRenderer
    }

    pub fn run(
        &self,
        map: Arc<RwLock<Map>>,
        base_state: Arc<Mutex<BaseState>>,
        robot_states: Arc<Mutex<Vec<RobotState>>>,
        running: Arc<AtomicBool>,
    ) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            if !running.load(Ordering::Relaxed) {
                break;
            }
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(_) = event::read()? {
                    break;
                }
            }

            terminal.draw(|frame| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(12),
                        Constraint::Length(3),
                    ])
                    .split(frame.area());

                frame.render_widget(
                    Paragraph::new("RESOURCE COLLECTION SIMULATION")
                        .style(
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                        .block(Block::default().borders(Borders::ALL)),
                    chunks[0],
                );

                frame.render_widget(self.render_map(&map, &robot_states), chunks[1]);
                frame.render_widget(self.render_counters(&base_state), chunks[2]);
            })?;
        }

        running.store(false, Ordering::Relaxed);
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn render_map(
        &self,
        map: &Arc<RwLock<Map>>,
        robot_states: &Arc<Mutex<Vec<RobotState>>>,
    ) -> Paragraph<'_> {
        let map = map.read().unwrap();
        let robots = robot_states.lock().unwrap();

        let robot_positions: HashMap<(usize, usize), RobotKind> = robots
            .iter()
            .map(|r| ((r.pos.x, r.pos.y), r.kind))
            .collect();

        let lines: Vec<Line> = (0..map.height)
            .map(|y| {
                let spans: Vec<Span> = (0..map.width)
                    .map(|x| {
                        if let Some(kind) = robot_positions.get(&(x, y)) {
                            let (symbol, color) = match kind {
                                RobotKind::Scout => ("x", Color::Red),
                                RobotKind::Collector => ("o", Color::Magenta),
                            };
                            Span::styled(symbol, Style::default().fg(color))
                        } else {
                            let (symbol, color) = match map.tiles[y][x] {
                                Tile::Empty => (".", Color::DarkGray),
                                Tile::Obstacle => ("O", Color::LightCyan),
                                Tile::Base => ("#", Color::LightGreen),
                                Tile::Resource(ResourceKind::Energy) => ("E", Color::Green),
                                Tile::Resource(ResourceKind::Crystal) => ("C", Color::LightMagenta),
                            };
                            Span::styled(symbol, Style::default().fg(color))
                        }
                    })
                    .collect();
                Line::from(spans)
            })
            .collect();

        Paragraph::new(lines).block(Block::default().title("Map").borders(Borders::ALL))
    }

    fn render_counters(&self, base_state: &Arc<Mutex<BaseState>>) -> Paragraph<'_> {
        let base = base_state.lock().unwrap();
        Paragraph::new(Line::from(vec![
            Span::styled("Energie: ", Style::default().fg(Color::Green)),
            Span::raw(base.total_energy.to_string()),
            Span::styled("   Cristaux: ", Style::default().fg(Color::LightMagenta)),
            Span::raw(base.total_crystals.to_string()),
        ]))
        .block(
            Block::default()
                .title("Ressources collectees (appuyez sur une touche pour quitter)")
                .borders(Borders::ALL),
        )
    }
}
