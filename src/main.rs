use crate::state::State;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    terminal, ExecutableCommand, Result,
};

mod menu;
mod state;

fn main() -> Result<()> {
    let mut state = State::init("menu.yaml");

    let mut stdout = std::io::stderr();
    stdout.execute(terminal::Clear(terminal::ClearType::All))?;
    stdout.execute(cursor::Hide)?;

    state.draw()?;

    loop {
        if let Event::Key(event) = read()? {
            // Reading input mode
            if let Some((ref _prefix, ref mut buf)) = state.reading() {
                match event.code {
                    KeyCode::Char(c) => buf.push(c),
                    KeyCode::Enter => {
                        let (prefix, buf) = state.reading().take().expect("unreachable");
                        println!("{}{}", prefix, buf);
                        std::process::exit(0);
                    }
                    _ => {}
                }
                // Don't continue to navigation mode
                continue;
            }
            // Normal navigation mode
            match event.code {
                KeyCode::Char(key) => state.handle_key(key)?,
                KeyCode::Backspace => {
                    // Only redraw if we actually changed menus
                    if state.pop_stack().is_some() {
                        state.draw()?;
                    }
                }
                _ => {
                    print!("unknown input {:?}\r\n", event);
                    terminal::disable_raw_mode()?;
                    std::process::exit(1)
                }
            }
        }
    }
}
