use std::env;
use std::fs;
use std::path::PathBuf;
use colored::Colorize;
use comfy_table::{Table, ContentArrangement};
use homedir::my_home;
use pyo3::prelude::*;
use pyo3::types::PyDict;

struct PluginInfo {
    module_path: String,
    name: String,
    description: String,
    version: String,
    error: Option<String>,
}

fn get_metadata_from_file(file_path: &PathBuf, module_path: &str) -> PluginInfo {
    let code = match fs::read_to_string(file_path) {
        Ok(code) => code,
        Err(e) => {
            return PluginInfo {
                module_path: module_path.to_string(),
                name: String::new(),
                description: String::new(),
                version: String::new(),
                error: Some(format!("Failed to read file: {}", e)),
            };
        }
    };

    // Use Python to extract metadata
    let result = Python::with_gil(|py| -> Result<PluginInfo, String> {
        let module_name = file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        let plugin = match PyModule::from_code(py, &code, file_path.to_str().unwrap_or(""), module_name) {
            Ok(p) => p,
            Err(e) => {
                return Err(format!("Load error: {}", e.value(py)));
            }
        };

        let config = match plugin.getattr("VALRADAR_CONFIG") {
            Ok(c) => c,
            Err(_) => {
                return Err("Missing VALRADAR_CONFIG".to_string());
            }
        };

        let config_dict = match config.extract::<&PyDict>() {
            Ok(d) => d,
            Err(_) => {
                return Err("VALRADAR_CONFIG is not a dict".to_string());
            }
        };

        let metadata = match config_dict.get_item("metadata") {
            Ok(Some(m)) => m,
            _ => {
                return Ok(PluginInfo {
                    module_path: module_path.to_string(),
                    name: module_name.to_string(),
                    description: "No description".to_string(),
                    version: "?".to_string(),
                    error: None,
                });
            }
        };

        let metadata_dict = match metadata.extract::<&PyDict>() {
            Ok(d) => d,
            Err(_) => {
                return Ok(PluginInfo {
                    module_path: module_path.to_string(),
                    name: module_name.to_string(),
                    description: "No description".to_string(),
                    version: "?".to_string(),
                    error: None,
                });
            }
        };

        let name = metadata_dict.get_item("name")
            .ok()
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or_else(|| module_name.to_string());
        
        let description = metadata_dict.get_item("description")
            .ok()
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "No description".to_string());
        
        let version = metadata_dict.get_item("version")
            .ok()
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "?".to_string());

        Ok(PluginInfo {
            module_path: module_path.to_string(),
            name,
            description,
            version,
            error: None,
        })
    });

    match result {
        Ok(info) => info,
        Err(e) => PluginInfo {
            module_path: module_path.to_string(),
            name: String::new(),
            description: String::new(),
            version: String::new(),
            error: Some(e),
        },
    }
}

fn find_plugins_in_dir(dir: &PathBuf, prefix: &str) -> Vec<PluginInfo> {
    let mut plugins = Vec::new();

    if !dir.exists() {
        return plugins;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return plugins,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        
        if path.is_file() && path.extension().map(|e| e == "py").unwrap_or(false) {
            let name = path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            
            // Skip __init__.py and similar
            if name.starts_with('_') {
                continue;
            }
            
            let module_path = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}.{}", prefix, name)
            };
            
            plugins.push(get_metadata_from_file(&path, &module_path));
        } else if path.is_dir() {
            // Recursively search subdirectories
            let subdir_name = path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            
            // Skip hidden directories
            if subdir_name.starts_with('.') || subdir_name.starts_with('_') {
                continue;
            }
            
            let subprefix = if prefix.is_empty() {
                subdir_name.to_string()
            } else {
                format!("{}.{}", prefix, subdir_name)
            };
            
            plugins.extend(find_plugins_in_dir(&path, &subprefix));
        }
    }

    plugins
}

pub fn list_plugins() {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let home_dir = my_home().ok().flatten().unwrap_or_else(|| PathBuf::from("."));

    let search_paths = vec![
        (current_dir.join("examples"), "examples"),
        (current_dir.join("modules"), "modules"),
        (home_dir.join(".valradar").join("modules"), "~/.valradar/modules"),
    ];

    let mut all_plugins = Vec::new();
    let mut searched_locations = Vec::new();

    for (path, display_name) in search_paths {
        if path.exists() {
            searched_locations.push(format!("{} ({})", display_name, path.display()));
            let prefix = display_name.split('/').last().unwrap_or(display_name);
            all_plugins.extend(find_plugins_in_dir(&path, prefix));
        }
    }

    if all_plugins.is_empty() {
        println!("{}", "No plugins found.".yellow());
        println!();
        println!("Searched locations:");
        for loc in &searched_locations {
            println!("  - {}", loc);
        }
        if searched_locations.is_empty() {
            println!("  (no plugin directories found)");
        }
        println!();
        println!("Create a new plugin with: valradar new <name>");
        return;
    }

    // Build and display the table
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Module", "Name", "Version", "Description"]);

    let mut error_plugins = Vec::new();

    for plugin in &all_plugins {
        if let Some(ref err) = plugin.error {
            error_plugins.push((plugin.module_path.clone(), err.clone()));
            continue;
        }

        table.add_row(vec![
            plugin.module_path.clone(),
            plugin.name.clone(),
            plugin.version.clone(),
            // Truncate long descriptions
            if plugin.description.len() > 50 {
                format!("{}...", &plugin.description[..47])
            } else {
                plugin.description.clone()
            },
        ]);
    }

    let valid_count = all_plugins.len() - error_plugins.len();
    
    println!("{}", "Available Plugins".bold());
    println!();
    
    if valid_count > 0 {
        println!("{}", table);
    }

    if !error_plugins.is_empty() {
        println!();
        println!("{} ({} plugin(s) failed to load):", "Errors".red().bold(), error_plugins.len());
        for (module, err) in &error_plugins {
            println!("  {} - {}", module.yellow(), err);
        }
    }

    println!();
    println!("Found {} plugin(s) in {} location(s)", all_plugins.len(), searched_locations.len());
    println!();
    println!("Run a plugin with: valradar run <module> [args...]");
}
