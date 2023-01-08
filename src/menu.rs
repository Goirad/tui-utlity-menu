use serde::{Deserialize, Deserializer};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Menu {
    title: String,
    message: String,
    // We use a Vec instead of a HashMap to preserve the ordering in the
    // menu definition
    entries: Vec<MenuEntry>,
}

impl Menu {
    pub fn get_action(&self, key: char) -> Option<&MenuAction> {
        self.entries
            .iter()
            .find(|e| e.hotkey == key)
            .map(|e| &e.action)
    }
}

impl Display for Menu {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "  {}\n\n", self.title)?;
        write!(f, "  {}\n\n", self.message)?;
        for entry in &self.entries {
            writeln!(f, "  {}", entry)?;
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for Menu {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut hotkeys = vec!['l', 'k', 'j', 'h', 'g', 'f', 'd', 's', 'a'];

        #[derive(Debug, Deserialize)]
        struct MenuEntryInner {
            hotkey: Option<char>,
            message: String,
            action: MenuAction,
        }

        #[derive(Deserialize)]
        struct MenuInner {
            title: String,
            message: String,
            entries: Vec<MenuEntryInner>,
        }

        fn push_entry(menu: &mut Menu, hotkeys: &mut Vec<char>, entry: MenuEntryInner) {
            let hotkey = match entry.hotkey {
                Some(key) => {
                    // Assert we don't have a hotkey collision
                    assert!(
                        !menu.entries.iter().map(|e| e.hotkey).any(|k| k == key),
                        "detected a hotkey collision while parsing menu \"{}\"",
                        menu.title
                    );

                    // Remove `key` from our pool if present
                    hotkeys.retain(|c| *c != key);
                    key
                }
                None => {
                    // If no hotkey is provided, assign a free hotkey from our
                    // internal pool
                    hotkeys.pop().unwrap_or_else(|| {
                        panic!(
                            "Menu \"{}\" ran out of hotkeys while parsing entries",
                            menu.title
                        )
                    })
                }
            };

            let MenuEntryInner {
                message, action, ..
            } = entry;

            menu.entries.push(MenuEntry {
                hotkey,
                message,
                action,
            });
        }

        let menu_inner = MenuInner::deserialize(deserializer)?;

        let MenuInner {
            title,
            message,
            entries,
        } = menu_inner;

        let mut menu = Menu {
            title,
            message,
            entries: Vec::new(),
        };

        for entry in entries {
            push_entry(&mut menu, &mut hotkeys, entry);
        }

        Ok(menu)
    }
}

#[derive(Debug, Deserialize)]
pub enum MenuAction {
    Terminal(String), // Return inner and exit
    SubMenu(Menu),    // go to another menu
    Prompt(Prompt),   // .0 is prompt, .1 is id prefix
}

#[derive(Debug, Deserialize)]
pub struct Prompt {
    pub prompt: String,
    pub prefix: String,
}

#[derive(Debug, Deserialize)]
pub struct MenuEntry {
    hotkey: char,
    message: String,
    action: MenuAction,
}

impl Display for MenuEntry {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "({}) - {:>3} - {}",
            self.hotkey,
            match self.action {
                MenuAction::SubMenu(..) => "[M]",
                _ => "",
            },
            self.message
        )
    }
}

// A path into a menu, represented by the series of keystrokes needed to get
// there
#[derive(Debug, Default)]
pub struct MenuStack(Vec<char>);

impl Display for MenuStack {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        // Looks like "a -> b -> c"
        let chain = self
            .0
            .iter()
            .map(|c| c.to_string())
            .reduce(|accum, new| format!("{} -> {}", accum, new))
            .unwrap_or_default();
        f.write_str(&chain)
    }
}

impl std::ops::Deref for MenuStack {
    type Target = Vec<char>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MenuStack {
    fn deref_mut(&mut self) -> &mut Vec<char> {
        &mut self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn menu_descent() {
        let yaml = r"
title: Menu1
message: Message1
entries:
- message: Message2
  hotkey: a
  action: !SubMenu
    title: Menu2
    message: Message3
    entries:
    - message: Message4
      hotkey: b
      action: !SubMenu
        title: Menu3
        message: Message4
        entries:
        - message: Message5
          hotkey: c
          action: !Terminal DESIRED
        - message: Message6
          hotkey: d
          action: !Prompt
            prompt: my prompt
            prefix: my_prefix
";
        macro_rules! expect {
            ($menu:ident, $exp:ident, $key:literal) => {
                match $menu.get_action($key) {
                    Some(MenuAction::$exp(inner)) => inner,
                    other => panic!(
                        "expected {} at '{}', go {:?}",
                        stringify!($exp),
                        $key,
                        other
                    ),
                }
            };
        }
        // We want to verify a -> b -> c
        let menu: Menu = serde_yaml::from_str(&yaml).expect("failed to parse menu file");
        let submenu = expect!(menu, SubMenu, 'a');
        let submenu = expect!(submenu, SubMenu, 'b');
        let terminal = expect!(submenu, Terminal, 'c');
        assert_eq!(terminal, "DESIRED");
        let prompt = expect!(submenu, Prompt, 'd');
        assert_eq!(prompt.prompt, "my prompt");
        assert_eq!(prompt.prefix, "my_prefix");
    }
}
