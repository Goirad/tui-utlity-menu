# TUI Utility Menu

This is a rust binary meant to allow for easy access to arbitrary actions on
your computer. It is intended to be used as a glue, turning user input into a
machine readable input into a bash script input, and from there you can do
anything you want.

Specifically, you can map a system keybind to open this as a modal window. In
your i3 config:
```
bindsym $mod+u exec alacritty --class "i3-floating" -e ~/.config/scripts/menu.sh
```

Check `examples/menu.sh` and `examples/menu.yaml` for some example files.

# Setup

1. Install the binary:
```bash
$ git clone git@github.com:Goirad/tui-utlity-menu.git
$ cd tui-utlity-menu
$ cargo install --path ./
```
2. Define your menu (take a look at `examples/menu.yaml`)
3. Define your glue script (again, take a look at `examples/menu.sh`)
4. Run your script:
```bash
$ ./examples/menu.sh ./examples/menu.yaml
```

# Config File Format
The config file is a yaml file.

### Top Level
This is the top level type for the config file. No hotkeys, and always has a
list of entries, which correspond to actions your menu can do.
```yaml
title: Menu # This is a menu header
message: This is what this menu is for # a longer description about this menu
hotkey: h # Optional hotkey, not applicable to top level menu
          # If no hotkey is provided, the home key rows are used from left to
          # right, e.g. a, s, d, f, etc
entries:
  - message: A short message # this should describe what this action does
    hotkey: h # Optional hotkey to trigger this entry's action
              # If no hotkey is provided, the home key rows are used from left to
              # right, e.g. a, s, d, f, etc
    action: !type # <type> is one of the following actions

```
## Actions
### Submenu
Use this to define a submenu, for example to group related actions.
```yaml
action: !SubMenu
  title: The menu title
  message: Change display settings # the menu's description
  entries:
  - message: description of this entry
    hotkey: h # optional hotkey
    action: <another action>
```

### Terminal
The most basic non-menu action, just echo a fixed string to stdout for
consumption by the glue script.
```yaml
action: !Terminal WEBSITE_GOOGLE
```

### Prompt
Like Terminal, but first query the user for some input to append to the prefix
```yaml
action: !Prompt
  prompt: "<FOO>-<BAR>:" # a hint for the user
  prefix: "RUST_ISSUE_"
```
