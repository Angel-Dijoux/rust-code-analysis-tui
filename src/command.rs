use ratatui::{
    layout::Constraint,
    style::{Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Cell, Row, Table},
};

use crate::{analysis, error::AppResult, navigator::FileNavigator};

pub trait Command {
    fn execute(
        &mut self,
        navigator: &mut FileNavigator,
        details: &mut Option<Table>,
        analysis: &mut Option<Table>,
    ) -> AppResult<()>;
}

pub struct AnalyzeCommand;

impl Command for AnalyzeCommand {
    fn execute(
        &mut self,
        navigator: &mut FileNavigator,
        details: &mut Option<Table>,
        analysis: &mut Option<Table>,
    ) -> AppResult<()> {
        if let Some(path) = navigator.selected() {
            if path.is_dir() {
                *details = None;
                *analysis = Some(analysis::analyze_directory(path)?);
            }
        }
        Ok(())
    }
}

pub struct SelectFileCommand;

impl Command for SelectFileCommand {
    fn execute(
        &mut self,
        navigator: &mut FileNavigator,
        details: &mut Option<Table>,
        _analysis: &mut Option<Table>,
    ) -> AppResult<()> {
        if let Some(path) = navigator.selected() {
            if !path.is_dir() {
                let table = Table::new(
                    vec![Row::new(vec![
                        Cell::from(Text::from("Path")),
                        Cell::from(Text::from(path.display().to_string())),
                    ])],
                    [Constraint::Percentage(30), Constraint::Percentage(70)],
                )
                .header(
                    Row::new(vec![Cell::from(Text::from("Path"))])
                        .style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .block(
                    Block::default()
                        .title("Selected Path")
                        .borders(Borders::ALL),
                );

                *details = Some(table);
            }
        }
        Ok(())
    }
}
