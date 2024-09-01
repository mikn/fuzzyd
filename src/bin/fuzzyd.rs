use fuzzyd::Fuzzyd;
use clap::{Parser, Subcommand};
use fuzzyd::sources::Source;
use fuzzyd::fuzzy::FuzzyItem;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "fuzzyd",
    author = "Mikael Knutsson <mikael.knutsson@gmail.com>",
    version,
    about = "A fast, efficient, and pluggable fuzzy finder for launching applications and executables",
    long_about = "fuzzyd is a terminal-based application launcher that uses fuzzy searching to quickly find and execute applications or commands. It supports multiple sources for executables and provides a customizable interface.",
    after_help = "For more information, visit: https://github.com/mikn/fuzzyd"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Sources to search (desktop, path, or both if not specified)
    #[arg(
        value_enum,
        help = "Specify which sources to search for executables",
        long_help = "Choose one or more sources to search for executables. If not specified, both desktop entries and PATH executables will be searched.",
        num_args = 0..,
    )]
    sources: Vec<Source>,

    /// Enable debug mode
    #[arg(
        short,
        long,
        help = "Run fuzzyd in debug mode",
        long_help = "Enable debug mode to display additional information during runtime, such as the number of items loaded and detailed search results."
    )]
    debug: bool,

    /// Specify a custom configuration file
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Use a custom configuration file",
        long_help = "Specify the path to a custom TOML configuration file. If not provided, fuzzyd will use the default configuration file at ~/.config/fuzzyd/config.toml"
    )]
    config: Option<PathBuf>,

    /// Launch the specified command directly
    #[arg(
        short,
        long,
        value_name = "COMMAND",
        help = "Launch a command directly without entering the interactive mode",
        long_help = "Specify a command to launch directly, bypassing the interactive fuzzy search interface. This is useful for scripting or quick launches from other applications."
    )]
    exec: Option<String>,

    /// Disable history
    #[arg(
        long,
        help = "Disable command history",
        long_help = "Disable the use of command history for ranking results. By default, history is enabled.",
    )]
    disable_history: bool,

    /// Specify a custom history file
    #[arg(
        long,
        value_name = "FILE",
        help = "Use a custom history file",
        long_help = "Specify the path to a custom history file. If not provided, fuzzyd will use the default history file at ~/.local/share/fuzzyd/fuzzyd.history"
    )]
    history_file: Option<PathBuf>,

    /// Dry run mode
    #[arg(
        long,
        help = "Print the command that would be run, but do not execute it",
        long_help = "Enable dry run mode to print the command that would be run, but do not execute it. This is useful for testing and debugging."
    )]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Write the default configuration file to the user's configuration directory
    Init,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(Commands::Init) = cli.command {
        write_default_config()?;
        return Ok(());
    }

    let sources = if cli.sources.is_empty() {
        vec![Source::Desktop, Source::Path]
    } else {
        cli.sources
    };

    let history_file = if cli.disable_history {
        None
    } else {
        cli.history_file
    };

    let mut fuzzyd = Fuzzyd::new(sources, cli.debug, cli.config, history_file, cli.dry_run)?;

    if let Some(command) = cli.exec {
        let item = FuzzyItem {
            display: command.clone(),
            exec: command,
            priority: 0,
            source_order: 0,
            description: String::new(),
            source_path: String::new(),
            search_desc: false,
            icon: String::new(),
        };
        fuzzyd.launch(&item)?;
    } else {
        fuzzyd.run()?;
    }

    Ok(())
}

fn write_default_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not determine config directory")?;
    let config_path = config_dir.join("fuzzyd").join("config.toml");

    let default_config = r###"
debug = false

[ui]
prompt = "#"
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
"###;

    fs::create_dir_all(config_path.parent().unwrap())?;
    fs::write(config_path.clone(), default_config)?;

    println!("Default configuration file written to {:?}", config_path);
    Ok(())
}