mod analysis;
mod command;
mod error;
mod metrics;
mod navigator;
mod ui;

use crate::command::{AnalyzeCommand, Command, SelectFileCommand};
use crate::error::AppResult;
use crate::navigator::FileNavigator;
use crate::ui::TerminalUI;
use crossterm::event::{self, Event, KeyCode};
use dirs::home_dir;
use std::env;
use std::path::Path;
use std::time::Duration;

fn run_app(path: String) -> AppResult<()> {
    let mut details = None;
    let mut analysis = None;

    let mut navigator = FileNavigator::new(path.as_ref())?;
    let mut ui = TerminalUI::new()?;
    let mut analyze_cmd = AnalyzeCommand;
    let mut select_cmd = SelectFileCommand;

    loop {
        ui.draw(&navigator, analysis.clone(), details.clone())?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => navigator.next(),
                    KeyCode::Up => navigator.previous(),
                    KeyCode::Enter => {
                        if let Some(path) = navigator.selected() {
                            let cmd: &mut dyn Command = if path.is_dir() {
                                &mut analyze_cmd
                            } else {
                                &mut select_cmd
                            };
                            cmd.execute(&mut navigator, &mut details, &mut analysis)?;
                        }
                    }
                    KeyCode::Esc => details = None,
                    _ => {}
                }
            }
        }
    }

    ui.cleanup()?;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = if args.len() > 1 {
        args[1].clone()
    } else {
        home_dir()
            .unwrap_or_else(|| Path::new("/tmp").to_path_buf())
            .to_str()
            .unwrap()
            .to_string()
    };

    if !Path::new(&path).exists() {
        eprintln!("Error: Path '{}' does not exist", path);
        std::process::exit(1);
    }
    if let Err(err) = run_app(path) {
        eprintln!("Application error: {}", err);
        std::process::exit(1);
    }
}
