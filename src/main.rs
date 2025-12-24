use std::collections::HashMap;
use std::env;
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};
use clap::{Parser, command};
use valradar::{Plugin, Orchestrator, utils};

#[derive(Debug, Parser)]
#[command(
    author = "Mainasara Tsowa (tsowamainasara@gmail.com)",
    version = "0.2.0",
    about = "Valradar is a high-performance, low-latency, and scalable data processing framework for OSINT, RECON, and a wide range of operations.",
    after_help = "MIT License (c) 2025 Mainasara Tsowa",
    long_about = "Valradar is a high-performance, low-latency, and scalable data processing framework designed for OSINT, RECON, and a wide range of operations. It provides a flexible plugin architecture that allows you to create custom data collection and processing pipelines."
)]
struct Args {
    #[arg(short = '!', long, long_help = "Enable debug mode", default_value = "false")]
    debug: bool,

    #[arg(short = 'm', long, long_help = "Maximum tasks to process (0 = unlimited)", default_value = "0")]
    max_tasks: u32,

    #[arg(short = 'c', long, long_help = "How many concurrent threads to use", default_value = "4")]
    concurrency: u32,

    #[arg(short = 'i', long, long_help = "Show plugin information", default_value = "false")]
    info: bool,

    #[arg(short = 'H', long = "plugin-help", long_help = "Show plugin-specific help and options", default_value = "false")]
    plugin_help: bool,

    #[arg(short = 'l', long, long_help = "Show license", default_value = "false")]
    license: bool,

    #[arg(help = "Plugin module name", default_value = "_")]
    plugin: String,

    #[arg(help = "Targets and plugin options (--option value)", last = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

/// Parse plugin arguments into targets and kwargs
/// Supports: --option value, --option=value formats
fn parse_plugin_args(args: &[String]) -> (Vec<String>, HashMap<String, String>) {
    let mut targets = Vec::new();
    let mut kwargs = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with("--") {
            let opt = arg.trim_start_matches("--");

            // Handle --option=value format
            if let Some(eq_pos) = opt.find('=') {
                let key = opt[..eq_pos].to_string();
                let value = opt[eq_pos + 1..].to_string();
                kwargs.insert(key, value);
            } else {
                // Handle --option value format
                let key = opt.to_string();
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    kwargs.insert(key, args[i + 1].clone());
                    i += 1;
                } else {
                    // Flag without value, treat as boolean true
                    kwargs.insert(key, "true".to_string());
                }
            }
        } else if arg.starts_with('-') && arg.len() == 2 {
            // Short option: -o value
            let key = arg.chars().nth(1).unwrap().to_string();
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                kwargs.insert(key, args[i + 1].clone());
                i += 1;
            } else {
                kwargs.insert(key, "true".to_string());
            }
        } else {
            // Regular argument (target)
            targets.push(arg.clone());
        }

        i += 1;
    }

    (targets, kwargs)
}

/// Print plugin-specific help
fn print_plugin_help(plugin: &Plugin, plugin_name: &str) {
    match plugin.get_info() {
        Ok(info) => {
            println!("{} v{}", info.name, info.version);
            if !info.description.is_empty() {
                println!("{}", info.description);
            }
            if !info.author.is_empty() {
                println!("Author: {}", info.author);
            }
            println!();
            println!("USAGE:");
            println!("    valradar {} <TARGET> [OPTIONS]", plugin_name);
            println!();

            if !info.options.is_empty() {
                println!("PLUGIN OPTIONS:");
                for opt in &info.options {
                    let req = if opt.required { " (required)" } else { "" };
                    let def = opt.default.as_ref()
                        .map(|d| format!(" [default: {}]", d))
                        .unwrap_or_default();

                    println!("    --{:<20} {}{}{}", opt.name, opt.help, req, def);
                }
                println!();
            }

            println!("EXAMPLES:");
            if info.options.iter().any(|o| o.name == "pattern") {
                println!("    valradar {} https://example.com --pattern \"emails=[a-z]+@[a-z]+\"", plugin_name);
            } else {
                println!("    valradar {} https://example.com", plugin_name);
            }
        }
        Err(e) => {
            println!("Failed to get plugin info: {}", e);
        }
    }
}

fn main() {
    let args = Args::parse();

    if args.license {
        utils::license::print_license();
        return;
    }

    if args.plugin == "_" {
        println!("Plugin module name is required");
        println!("Usage: valradar <plugin> [targets...] [--options]");
        println!("Example: valradar modules.web.emails https://example.com");
        println!();
        println!("Use -H or --plugin-help with a plugin name to see plugin-specific options");
        return;
    }

    let plugin_name = args.plugin.clone();
    let parts = plugin_name.split('.').collect::<Vec<&str>>();
    let plugin_module_name = match parts.last().cloned() {
        Some(name) => name,
        None => "main".into(),
    };

    let mut plugin_path = parts.join("/") + ".py";
    if let Some(path) = utils::module::search_module(&plugin_path) {
        plugin_path = path;
    } else {
        println!("Plugin '{}' not found", plugin_name);
        return;
    }

    let plugin = Plugin::new(plugin_module_name.to_string(), plugin_path.to_string());

    if args.debug {
        unsafe {
            env::set_var("VALRADAR_DEBUG", "1");
        }
    }

    // Show plugin-specific help
    if args.plugin_help {
        print_plugin_help(&plugin, &plugin_name);
        return;
    }

    // Get and display metadata
    let metadata = match plugin.get_metadata() {
        Ok(metadata) => metadata,
        Err(e) => {
            println!("Failed to get metadata: {}", e);
            return;
        }
    };

    if args.info {
        println!("{}", metadata);
        return;
    }

    // Parse plugin arguments
    let (targets, kwargs) = parse_plugin_args(&args.args);

    // Validate targets
    if targets.is_empty() {
        println!("No targets provided");
        println!("Usage: valradar {} <target> [target...] [--options]", plugin_name);
        println!();
        println!("Use -H or --plugin-help to see plugin-specific options");
        return;
    }

    utils::print_banner(&metadata);

    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(std::time::Duration::from_millis(100));
    bar.set_message("Initializing plugin...");

    // Instantiate the plugin (calls setup())
    let instance = match plugin.instantiate() {
        Ok(instance) => {
            utils::debug("Plugin instantiated and setup() called");
            instance
        }
        Err(e) => {
            bar.finish_and_clear();
            println!("Plugin instantiation failed: {}", e);
            return;
        }
    };

    bar.set_message("Starting scan...");

    // Create orchestrator with the plugin instance
    let mut orchestrator = Orchestrator::new(plugin, instance, args.concurrency as usize);

    // Set max tasks if specified
    if args.max_tasks > 0 {
        orchestrator.set_max_tasks(args.max_tasks as usize);
    }

    // Set plugin kwargs
    if !kwargs.is_empty() {
        utils::debug(&format!("Plugin options: {:?}", kwargs));
        orchestrator.set_kwargs(kwargs);
    }

    let start_time = Instant::now();

    // Run with initial targets
    let results = match orchestrator.run(targets) {
        Ok(results) => {
            utils::debug(&format!("Scan completed with {} results", results.len()));
            results
        }
        Err(e) => {
            bar.finish_and_clear();
            println!("Scan failed: {}", e);
            return;
        }
    };

    let elapsed = start_time.elapsed();
    let tasks_processed = orchestrator.tasks_processed();
    let targets_visited = orchestrator.targets_visited();

    bar.set_style(ProgressStyle::with_template("{prefix} {msg}").unwrap());
    bar.set_prefix("✅");
    bar.set_message(format!(
        "Completed: {} results, {} tasks processed, {} unique targets in {:.2}s",
        results.len(),
        tasks_processed,
        targets_visited,
        elapsed.as_secs_f64()
    ));
    bar.finish();

    // Display results
    println!("{}", utils::ProcessedData::new(results));
}
