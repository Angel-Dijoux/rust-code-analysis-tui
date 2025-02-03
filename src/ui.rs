use crate::error::{AppError, AppResult};
use crate::navigator::FileNavigator;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::widgets::Table;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io;

pub struct TerminalUI {
    pub terminal: Terminal<CrosstermBackend<io::Stdout>>,
    pub list_state: ListState,
}

impl TerminalUI {
    pub fn new() -> AppResult<Self> {
        crossterm::terminal::enable_raw_mode()
            .map_err(|e| AppError::TerminalError(format!("Failed to enable raw mode: {}", e)))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| {
            AppError::TerminalError(format!("Failed to enter alternate screen: {}", e))
        })?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)
            .map_err(|e| AppError::TerminalError(format!("Terminal init error: {}", e)))?;
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Ok(Self {
            terminal,
            list_state,
        })
    }

    pub fn draw(
        &mut self,
        navigator: &FileNavigator,
        analysis: Option<Table>,
        detail: Option<Table>,
    ) -> AppResult<()> {
        self.list_state.select(Some(navigator.selected_index));
        self.terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(f.area());

                if let Some(analysis_table) = analysis {
                    f.render_widget(analysis_table, chunks[0]);
                } else {
                    let empty_paragraph = Paragraph::new("No analysis result available.")
                        .block(Block::default().borders(Borders::ALL).title("Analysis"));
                    f.render_widget(empty_paragraph, chunks[0]);
                }

                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                    .split(chunks[1]);

                let items: Vec<ListItem> = navigator
                    .entries
                    .iter()
                    .map(|path| {
                        let name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown");
                        ListItem::new(name)
                    })
                    .collect();
                let list = List::new(items)
                    .block(Block::default().borders(Borders::ALL).title("Files"))
                    .highlight_style(
                        Style::default()
                            .bg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(">> ");
                f.render_stateful_widget(list, right_chunks[0], &mut self.list_state);

                if let Some(detail_table) = detail {
                    f.render_widget(detail_table, right_chunks[1]);
                } else {
                    let empty_paragraph = Paragraph::new("No detail selected.")
                        .block(Block::default().borders(Borders::ALL).title("Details"));
                    f.render_widget(empty_paragraph, right_chunks[1]);
                }
            })
            .map_err(|e| AppError::TerminalError(format!("UI draw error: {}", e)))?;
        Ok(())
    }

    pub fn cleanup(&mut self) -> AppResult<()> {
        crossterm::terminal::disable_raw_mode()
            .map_err(|e| AppError::TerminalError(format!("Failed to disable raw mode: {}", e)))?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen).map_err(|e| {
            AppError::TerminalError(format!("Failed to leave alternate screen: {}", e))
        })?;
        self.terminal
            .show_cursor()
            .map_err(|e| AppError::TerminalError(format!("Failed to show cursor: {}", e)))?;
        Ok(())
    }
}
