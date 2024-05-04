use std::io::Write;
use std::env;
use std::path::Path;

use crate::state::State;

use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyModifiers},
    style,
    terminal,
    ExecutableCommand,
    Result,
};

mod menu;
mod state;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(args.get(1).expect("missing config file path"));
    let mut state = State::init(path);

    let mut stdout = std::io::stderr();
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    stdout.execute(cursor::Hide)?;

    state.draw()?;

    loop {
        if let Event::Key(event) = read()? {
            // Always bail on Ctrl + C
            if event.modifiers.contains(KeyModifiers::CONTROL) && event.code == KeyCode::Char('c') {
                terminal::disable_raw_mode()?;
                stdout.execute(cursor::Show)?;
                stdout.execute(cursor::EnableBlinking)?;
                std::process::exit(137);
            }
            // Reading input mode
            if let Some((ref _prefix, ref mut buf)) = state.reading() {
                stdout.execute(cursor::Show)?;
                stdout.execute(cursor::EnableBlinking)?;
                match event.code {
                    KeyCode::Char(c) => {
                        buf.push(c);
                        stdout.execute(style::Print(&c))?;
                        stdout.flush()?;
                    }
                    KeyCode::Backspace => {
                        if buf.pop().is_some() {
                            stdout.execute(cursor::MoveLeft(1))?;
                            stdout.execute(style::Print(" "))?;
                            stdout.execute(cursor::MoveLeft(1))?;
                            stdout.flush()?;
                        }
                    }
                    KeyCode::Esc => {
                        *state.reading() = None;
                        stdout.execute(cursor::Hide)?;
                        stdout.execute(cursor::DisableBlinking)?;
                        state.draw()?;
                    }
                    KeyCode::Enter => {
                        let (prefix, buf) = state.reading().take().expect("unreachable");
                        println!("{}{}", prefix, buf);
                        terminal::disable_raw_mode()?;
                        std::process::exit(0);
                    }
                    _ => {}
                }
                // Don't continue to navigation mode
                continue;
            }
            // Normal navigation mode
            match event.code {
                KeyCode::Backspace => {
                    // Only redraw if we actually changed menus
                    if state.pop_stack().is_some() {
                        state.draw()?;
                    }
                }
                KeyCode::Esc => {
                    match state.pop_stack() {
                        Some(_) => state.draw()?,
                        None => {
                            terminal::disable_raw_mode()?;
                            stdout.execute(cursor::Show)?;
                            stdout.execute(cursor::EnableBlinking)?;
                            std::process::exit(0);
                        }
                    }

                }
                KeyCode::Char(key) => state.handle_key(key)?,
                _ => {
                    println!("unknown input {:?}", event);
                    terminal::disable_raw_mode()?;
                    std::process::exit(1)
                }
            }
        }
    }
}
