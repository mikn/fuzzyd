# fuzzyd

fuzzyd is a fast, efficient, and pluggable fuzzy finder for launching applications and executables in Unix-like environments. Even with 1000s of entries in your PATH, it should start in less than 50ms. This project was almost completely written with the assistance of AI code generation by me who has very limited experience with Rust.

## Features

- Fuzzy search through desktop entries and PATH executables
- Terminal-based user interface with real-time results
- Efficient scoring algorithm for accurate matches
- Launches applications using systemd
- Supports XDG desktop entries
- Command history navigation
- Pluggable architecture for easy extension
- Configurable via TOML configuration file
- Customizable prompt and highlight color
- Dry run mode for testing commands without execution

## Pluggable Architecture

fuzzyd is designed with a pluggable architecture, making it easy to extend and customize:

- **Sources**: Add new sources for executable items by implementing the `SourceFinder` trait.
- **UI**: Create alternative user interfaces by implementing the `UI` trait.
- **Launcher**: Customize the launch mechanism by implementing the `Launcher` trait.
- **Scoring**: Modify the scoring algorithm in `src/fuzzy/scorer.rs` to change how matches are ranked.

This modular design allows for easy addition of new features and adaptation to different environments.

## Installation

To install fuzzyd, you need to have Rust and Cargo installed on your system. Then, follow these steps:

1. Clone the repository:
   ```
   git clone https://github.com/mikn/fuzzyd.git
   cd fuzzyd
   ```

2. Build the project:
   ```
   cargo build --release
   ```

3. The binary will be available at `target/release/fuzzyd`

## Usage

Run fuzzyd from the terminal:

```
./fuzzyd [sources]
```

Sources:

- `desktop`: XDG desktop entries
- `path`: Executables in PATH


- If no sources are specified, both desktop entries and PATH executables are searched.
- To search only desktop entries: `./fuzzyd desktop`
- To search only PATH executables: `./fuzzyd path`
- To search both: `./fuzzyd`

- Type to search for applications or executables
- Use arrow keys to navigate through results
- Press Enter to launch the selected item
- Press Esc to clear the query or exit if the query is empty
- Use Ctrl+P and Ctrl+N to navigate through command history

### Debug Mode

To run fuzzyd in debug mode, use the `--debug` flag:

```
./fuzzyd --debug
```

### Dry Run Mode

To run fuzzyd in dry run mode, use the `--dry-run` flag:

```
./fuzzyd --dry-run
```

This will print the commands that would be executed without actually running them.


## Configuration

fuzzyd can be configured using a TOML file located at `~/.config/fuzzyd/config.toml`. The following options are currently available:

```toml
debug = false
[ui]
prompt = "# "
highlight_color = "green"
[history]
enabled = true
file = "~/.local/share/fuzzyd/fuzzyd.history"
[systemd_run]
parameters = [
    "--quiet",
    "--user",
    "--property=EnvironmentFile=-$HOME/.config/sway/env",
    "--slice",
    "app.slice"
]
```

## TODO

- [x] Selectable sources
- [x] History-influenced ranking of results
- [x] Icon per source
- [x] Put the description of the application from the desktop file at the top (if it exists)
- [ ] Customizable keybindings
- [ ] Plugin system for additional sources
- [ ] Add in padding around the search

This should cover all the features and usage instructions for fuzzyd. Let me know if you need any more details!