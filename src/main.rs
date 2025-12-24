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

    #[arg(short = 'l', long, long_help = "Show license", default_value = "false")]
    license: bool,

    #[arg(help = "Plugin module name", default_value = "_")]
    plugin: String,

    #[arg(help = "Target arguments for the plugin (URLs, IPs, etc.)", last = true)]
    args: Vec<String>,
}

fn main() {
    let args = Args::parse();

    if args.license {
        utils::license::print_license();
        return;
    }

    if args.plugin == "_" {
        println!("Plugin module name is required");
        println!("Usage: valradar <plugin> [targets...]");
        println!("Example: valradar modules.web.email https://example.com");
        return;
    }

    let (plugin_name, plugin_args) = (args.plugin, args.args);
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

    // Validate targets
    if plugin_args.is_empty() {
        println!("No targets provided");
        println!("Usage: valradar {} <target> [target...]", plugin_name);
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

    let start_time = Instant::now();

    // Run with initial targets
    let results = match orchestrator.run(plugin_args) {
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
