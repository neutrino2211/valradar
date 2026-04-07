use std::env;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use valradar::{Plugin, Orchestrator, utils};

pub fn run(
    plugin_name: &str,
    plugin_args: Vec<String>,
    depth: u32,
    concurrency: u32,
    debug: bool,
    info: bool,
) {
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

    if debug {
        unsafe {
            env::set_var("VALRADAR_DEBUG", "1");
        }
    }

    let metadata = match plugin.get_metadata() {
        Ok(metadata) => metadata,
        Err(e) => {
            println!("Failed to get metadata: {}", e);
            return;
        },
    };

    if info {
        println!("{}", metadata);
        return;
    }

    utils::print_banner(&metadata);

    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(Duration::from_millis(100));
    bar.set_message("Initializing plugin...");

    // Create and initialize the orchestrator
    let mut orchestrator = Orchestrator::new(plugin, concurrency as usize);

    // Orchestrator depth
    let mut depth = depth;
    
    match orchestrator.init(&plugin_args) {
        Ok(_) => {
            utils::debug("Plugin initialized");
        },
        Err(e) => {
            println!("Plugin initialization failed: {}", e);
            return;
        },
    };

    let mut all_results: Vec<utils::ExecutionContext> = vec![];

    while depth > 0 {
        bar.set_message(format!("Collecting at depth [{}/{}] with {} results collected", concurrency - depth + 1, concurrency, all_results.len()));
        // Run the orchestrator to process all current data
        let results = match orchestrator.run() {
            Ok(results) => {
                utils::debug(&format!("Collecting completed with {} results", results.len()));
                results
            },
            Err(e) => {
                println!("Collecting failed: {}", e);    
                vec![]
            }
        };

        all_results.extend(results.clone());
        orchestrator.set_data_queue(results.clone());

        // Decrement the depth
        depth -= 1;
    }

    bar.set_message(format!("Collected {} results", all_results.len()));
    bar.set_prefix("✅");
    bar.finish();

    let plugin = orchestrator.relinquish_plugin();

    let processing_bar = ProgressBar::new(all_results.len().try_into().unwrap());
    processing_bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:80.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-"));
    processing_bar.set_message("Processing results...");

    let mut processing_results: Vec<utils::ProcessingResult> = vec![];
    for result in all_results {
        let processing_result = plugin.process_data(&result);
        processing_bar.inc(1);
        match processing_result {
            Ok(processing_result) => processing_results.push(processing_result),
            Err(e) => {
                utils::debug(&format!("Skipped processing result: {}", e));
            }
        }
    }

    processing_bar.set_message("Processing completed");
    processing_bar.finish();

    println!("{}", utils::ProcessedData(processing_results));
}
