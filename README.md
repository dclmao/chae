# Chae

A local GNOME appearance manager built with Rust, GTK4, and libadwaita.

## Features

- Lists installed GNOME Shell themes, icon themes, and cursor themes from local system and user directories.
- Applies icon and cursor themes through GNOME desktop settings.
- Offers a Default option in each category and hides the `Hicolor` icon theme from the choices.
- Applies Shell themes when the GNOME User Themes extension exposes its settings.
- Does not download themes or connect to a remote catalog.

## Build and run

You need Rust/Cargo and the GTK4 and libadwaita development packages installed.

```sh
cargo run
```

Shell theme application requires the GNOME Shell User Themes extension. Theme changes may not take effect everywhere until the affected applications or GNOME session are restarted.

## Local search paths

Shell themes are scanned in `~/.local/share/themes`, `~/.themes/`, `/usr/local/share/themes`, and `/usr/share/themes`. Application icons and cursor themes are both discovered in `~/.icons`, `~/.local/share/icons`, `/usr/local/share/icons`, and `/usr/share/icons`. A theme directory with an `index.theme` file and a `.cursor` marker file appears only under Mouse cursors; unmarked icon themes appear only under Application icons. The app only enumerates local directories; it does not install, modify, or remove theme files.
