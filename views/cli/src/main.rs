use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Position},
    widgets::{Block, Paragraph},
};

const LOGO: &str = r#"   █████████
  ███░░░░░███
 ███     ░░░   ██████   ██████  ████████
░███          ███░░███ ███░░███░░███░░███
░███         ░███ ░███░███████  ░███ ░░░
░░███     ███░███ ░███░███░░░   ░███
 ░░█████████ ░░██████ ░░██████  █████
  ░░░░░░░░░   ░░░░░░   ░░░░░░  ░░░░░"#;

struct App {
    input: String,
}

impl App {
    fn new() -> Self {
        Self {
            input: String::new(),
        }
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            let Event::Key(key) = event::read()? else {
                continue;
            };

            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Esc => return Ok(()),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(());
                }
                KeyCode::Char(character) => self.input.push(character),
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Enter => submit_input(&self.input),
                _ => {}
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let [logo_area, input_row, help_area, _] = Layout::vertical([
            Constraint::Length(LOGO.lines().count() as u16),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(frame.area());

        frame.render_widget(Paragraph::new(LOGO), logo_area);

        let [_, input_area, _] = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .areas(input_row);

        let available_width = input_area.width.saturating_sub(2) as usize;
        let input_width = self.input.chars().count();
        let horizontal_scroll =
            input_width.saturating_sub(available_width.saturating_sub(1)) as u16;

        frame.render_widget(
            Paragraph::new(self.input.as_str())
                .scroll((0, horizontal_scroll))
                .block(Block::bordered().title("Input")),
            input_area,
        );
        frame.render_widget(
            Paragraph::new("Enter: submit  •  Esc or Ctrl+C: quit"),
            help_area,
        );

        if input_area.width > 2 && input_area.height > 2 {
            frame.set_cursor_position(Position::new(
                input_area.x + 1 + (input_width as u16).saturating_sub(horizontal_scroll),
                input_area.y + 1,
            ));
        }
    }
}

pub fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::new().run(terminal))
}

/// Hook up submitted input here.
fn submit_input(input: &str) {
    dotenvy::dotenv().ok();

    let storage = JsonStorage {
        path: config::data_dir(),
    };

    let api_key = SecretString::from(
        std::env::var(config::AUTH_TOKEN_ENV_VAR)
            .unwrap_or_else(|_| panic!("{} must be set", config::AUTH_TOKEN_ENV_VAR)),
    );

    let connection = BibleAPIConnection {
        api_key,
        url: "https://api.esv.org/v3/passage/text/".to_string(),
    };

    let query: EsvResponse = connection.request_verse(input).await;
    let test_memory: Memory = query.into();
    storage.store(&test_memory).expect("Failed to store memory");

    let loaded = storage
        .read(test_memory.verse_header.clone())
        .expect("Failed to read memory back");
    dbg!(loaded);
}
