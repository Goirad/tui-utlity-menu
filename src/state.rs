use crossterm::{cursor, style, terminal, ExecutableCommand, QueueableCommand, Result};
use std::io::Write;
use std::time::Instant;

use crate::menu::{Menu, MenuAction, MenuStack};

pub struct State {
    menu: Menu,
    stack: MenuStack,
    load_time_reading: u128, // micros to read yaml
    load_time_parsing: u128, // micros to parse yaml
    // A buffer for reading user input, .0 is the prefix we'll be prepending
    // to the read buffer before writing to stdout
    reading: Option<(String, String)>,
}

impl State {
    pub fn init<F: AsRef<str>>(file: F) -> Self {
        // Load the file
        let start = Instant::now();
        let model = std::fs::read_to_string(file.as_ref()).expect("failed to read menu file");
        let load_time_reading = start.elapsed().as_micros();

        // Parse the file
        let start = Instant::now();
        let menu: Menu = serde_yaml::from_str(&model).expect("failed to parse menu file");
        let load_time_parsing = start.elapsed().as_micros();

        Self {
            menu,
            stack: Default::default(),
            load_time_reading,
            load_time_parsing,
            reading: None,
        }
    }

    pub fn reading(&mut self) -> &mut Option<(String, String)> {
        &mut self.reading
    }

    // Push a submenu onto the stack
    fn push_stack(&mut self, key: char) {
        // TODO verify `key` actually corresponds to a menu
        self.stack.push(key);
    }

    // Pop a submenu from the stack
    pub fn pop_stack(&mut self) -> Option<char> {
        self.stack.pop()
    }

    // Find a submenu by walking our menu stack
    fn current_menu(&self) -> &Menu {
        let mut current = &self.menu;
        for key in self.stack.iter() {
            // NOTE we know this is valid because we check when pushing
            // onto self.stack that `key` corresponds to a valid menu
            match current.get_action(*key) {
                Some(MenuAction::SubMenu(menu)) => current = menu,
                _ => unreachable!(),
            }
        }
        current
    }

    pub fn handle_key(&mut self, key: char) -> Result<()> {
        let mut out = std::io::stderr();

        // Bail early if we don't have an entry for `key`
        let action = match self.current_menu().get_action(key) {
            Some(action) => action,
            None => return Ok(()),
        };

        match action {
            MenuAction::Terminal(message) => {
                terminal::disable_raw_mode()?;
                println!("{}", message);
                out.flush()?;
                std::process::exit(0);
            }
            MenuAction::SubMenu(_submenu) => {
                self.push_stack(key);
                self.draw()?;
            }
            MenuAction::Prompt(prompt) => {
                terminal::disable_raw_mode()?;

                // Draw the prompt
                out.execute(cursor::MoveTo(2, 17))?;
                out.execute(style::Print(&prompt.prompt))?;
                out.execute(cursor::MoveTo(2, 18))?;
                out.execute(cursor::Show)?;
                out.flush()?;

                // Initialize a buffer for collecting user input
                self.reading = Some((prompt.prefix.to_owned(), String::new()));
            }
        }
        Ok(())
    }

    // Find the menu corresponding to the menu stack, and draw it.
    pub fn draw(&self) -> Result<()> {
        // We use stderr so that capturing stdout from bash stays easy
        let mut stdout = std::io::stderr();

        // Clear the terminal and get out of raw mode for drawing
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        terminal::disable_raw_mode()?;

        stdout
            .queue(cursor::MoveTo(2, 1))?
            .queue(style::Print(&self.stack))?
            .queue(cursor::MoveTo(0, 3))?
            .queue(style::Print(self.current_menu()))?
            .queue(cursor::MoveTo(44, 20))?
            .queue(style::Print(format!("{:>3}us", self.load_time_reading)))?
            .queue(cursor::MoveTo(50, 20))?
            .queue(style::Print(format!("{:>3}us", self.load_time_parsing)))?;

        stdout.flush()?;
        terminal::enable_raw_mode()?;
        Ok(())
    }
}
