use std::error::Error;
use std::io;

use coer_controller::Coer;
use coer_esv::EsvClient;
use coer_model::{Memory, Storage, VerseSource};
use coer_storage::{JsonStorage, data_dir};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Position},
    widgets::{Block, Paragraph, Wrap},
};

const LOGO: &str = r#"   █████████
  ███░░░░░███
 ███     ░░░   ██████   ██████  ████████
░███          ███░░███ ███░░███░░███░░███
░███         ░███ ░███░███████  ░███ ░░░
░░███     ███░███ ░███░███░░░   ░███
 ░░█████████ ░░██████ ░░██████  █████
  ░░░░░░░░░   ░░░░░░   ░░░░░░  ░░░░░"#;

struct App<S, V> {
    input: String,
    status: String,
    showing: Option<Memory>,
    coer: Coer<S, V>,
    /// The TUI loop is synchronous but fetching is not, so the view owns a
    /// runtime and blocks on it. See the note in `submit`.
    runtime: tokio::runtime::Runtime,
}

impl<S: Storage, V: VerseSource> App<S, V> {
    fn new(coer: Coer<S, V>, runtime: tokio::runtime::Runtime) -> Self {
        let status = match coer.memories() {
            Ok(refs) if refs.is_empty() => "Nothing saved yet. Type a reference.".to_string(),
            Ok(refs) => format!("{} saved: {}", refs.len(), join_refs(&refs)),
            Err(e) => describe(&e),
        };

        Self {
            input: String::new(),
            status,
            showing: None,
            coer,
            runtime,
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
                KeyCode::Enter => self.submit(terminal)?,
                _ => {}
            }
        }
    }

    /// The one place the view asks the application to do something.
    ///
    /// `block_on` freezes the UI for the length of the request. Acceptable while
    /// a fetch is the only action; the fix when it stops being acceptable is to
    /// `runtime.spawn` the call and receive the result over an mpsc channel, so
    /// the draw loop keeps running.
    fn submit(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let query = self.input.trim().to_string();
        if query.is_empty() {
            return Ok(());
        }

        // Repaint before blocking, so the user sees why nothing is responding.
        self.status = format!("Fetching {query}...");
        terminal.draw(|frame| self.draw(frame))?;

        match self.runtime.block_on(self.coer.add_passage(&query)) {
            Ok(memory) => {
                self.status = format!("Saved {}", memory.reference());
                self.showing = Some(memory);
                self.input.clear();
            }
            Err(e) => self.status = describe(&e),
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let [logo_area, input_row, help_area, status_area, passage_area] = Layout::vertical([
            Constraint::Length(LOGO.lines().count() as u16),
            Constraint::Length(3),
            Constraint::Length(1),
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
                .block(Block::bordered().title("Passage")),
            input_area,
        );
        frame.render_widget(
            Paragraph::new("Enter: fetch and save  •  Esc or Ctrl+C: quit"),
            help_area,
        );
        frame.render_widget(Paragraph::new(self.status.as_str()), status_area);

        if let Some(memory) = &self.showing {
            frame.render_widget(
                Paragraph::new(render(memory))
                    .wrap(Wrap { trim: true })
                    .block(Block::bordered().title(memory.reference().as_str().to_string())),
                passage_area,
            );
        }

        if input_area.width > 2 && input_area.height > 2 {
            frame.set_cursor_position(Position::new(
                input_area.x + 1 + (input_width as u16).saturating_sub(horizontal_scroll),
                input_area.y + 1,
            ));
        }
    }
}

/// Formatting for humans happens here and nowhere else.
fn render(memory: &Memory) -> String {
    let mut out = String::new();
    if !memory.passage.heading.is_empty() {
        out.push_str(&memory.passage.heading);
        out.push_str("\n\n");
    }
    for (number, text) in &memory.passage.verses {
        out.push_str(&format!("{number}. {text}\n"));
    }
    out
}

fn join_refs(refs: &[coer_model::VerseRef]) -> String {
    refs.iter()
        .map(|r| r.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// `#[source]` on the port errors builds a chain; this unwinds it so the user
/// sees "the storage backend failed: permission denied" instead of just the first half.
fn describe(error: &dyn Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        message.push_str(&format!(": {cause}"));
        source = cause.source();
    }
    message
}

fn main() -> Result<(), Box<dyn Error>> {
    // Process setup is the view's job — nothing in crates/ reads .env.
    dotenvy::dotenv().ok();

    let runtime = tokio::runtime::Runtime::new()?;

    // The only place in the whole workspace where concrete backends are named.
    let coer = Coer::new(JsonStorage::new(data_dir()), EsvClient::from_env()?);

    ratatui::run(|terminal| App::new(coer, runtime).run(terminal))?;
    Ok(())
}
