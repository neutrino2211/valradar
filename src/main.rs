mod commands;

use clap::{Parser, Subcommand};
use valradar::utils;

#[derive(Debug, Parser)]
#[command(
    author = "Mainasara Tsowa (tsowamainasara@gmail.com)",
    version = "0.1.0",
    about = "Valradar is a high-performance, low-latency, and scalable data processing framework for OSINT, RECON, and a wide range of operations.",
    after_help = "MIT License (c) 2025 Mainasara Tsowa",
    long_about = "Valradar is a high-performance, low-latency, and scalable data processing framework designed for OSINT, RECON, and a wide range of operations. It provides a flexible plugin architecture that allows you to create custom data collection and processing pipelines."
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    // Legacy flags for backward compatibility with `valradar <plugin>`
    #[arg(short = '!', long, long_help = "Enable debug mode", default_value = "false", global = true)]
    debug: bool,

    #[arg(short = 'l', long, long_help = "Show license", default_value = "false")]
    license: bool,

    // Allow positional plugin name for backward compatibility
    #[arg(help = "Plugin module name (legacy usage)", hide = true)]
    plugin: Option<String>,

    #[arg(help = "Arguments for the plugin (legacy usage)", last = true, hide = true)]
    args: Vec<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run a plugin
    #[command(alias = "r")]
    Run {
        /// Plugin module name (e.g., examples.emails)
        plugin: String,

        #[arg(short = 'd', long, long_help = "How many recursive calls to make", default_value = "1")]
        depth: u32,

        #[arg(short = 'c', long, long_help = "How many concurrent threads to use", default_value = "4")]
        concurrency: u32,

        #[arg(short = 'i', long, long_help = "Show plugin information", default_value = "false")]
        info: bool,

        /// Arguments for the plugin
        #[arg(last = true)]
        args: Vec<String>,
    },

    /// Create a new plugin from template
    #[command(alias = "n")]
    New {
        /// Name of the new plugin
        name: String,

        /// Output path (default: current directory)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// List available plugins
    #[command(alias = "ls")]
    List,

    /// Install plugin dependencies
    #[command(alias = "i")]
    Install {
        /// Plugin module name (e.g., examples.emails)
        plugin: String,
    },

    /// Validate plugin structure
    Doctor {
        /// Plugin module name (e.g., examples.emails)
        plugin: String,
    },
}

fn main() {
    let args = Args::parse();

    if args.license {
        utils::license::print_license();
        return;
    }

    match args.command {
        Some(Commands::Run { plugin, depth, concurrency, info, args: plugin_args }) => {
            commands::run::run(&plugin, plugin_args, depth, concurrency, args.debug, info);
        }
        Some(Commands::New { name, path }) => {
            commands::new::new_plugin(&name, path.as_deref());
        }
        Some(Commands::List) => {
            commands::list::list_plugins();
        }
        Some(Commands::Install { plugin }) => {
            commands::install::install_dependencies(&plugin);
        }
        Some(Commands::Doctor { plugin }) => {
            commands::doctor::doctor(&plugin);
        }
        None => {
            // Legacy mode: if plugin is provided without subcommand
            if let Some(plugin_name) = args.plugin {
                if plugin_name == "_" || plugin_name.is_empty() {
                    // Show help
                    println!("Usage: valradar <COMMAND>");
                    println!();
                    println!("Commands:");
                    println!("  run      Run a plugin");
                    println!("  new      Create a new plugin from template");
                    println!("  list     List available plugins");
                    println!("  install  Install plugin dependencies");
                    println!("  doctor   Validate plugin structure");
                    println!();
                    println!("Run 'valradar --help' for more information.");
                } else {
                    // Legacy: run plugin directly
                    commands::run::run(&plugin_name, args.args, 1, 4, args.debug, false);
                }
            } else {
                // Show help
                println!("Usage: valradar <COMMAND>");
                println!();
                println!("Commands:");
                println!("  run      Run a plugin");
                println!("  new      Create a new plugin from template");
                println!("  list     List available plugins");
                println!("  install  Install plugin dependencies");
                println!("  doctor   Validate plugin structure");
                println!();
                println!("Run 'valradar --help' for more information.");
            }
        }
    }
}
