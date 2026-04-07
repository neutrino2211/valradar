use std::fs;
use std::process::Command;
use colored::Colorize;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use crate::utils;

pub fn install_dependencies(plugin_name: &str) {
    // Find the plugin file
    let parts = plugin_name.split('.').collect::<Vec<&str>>();
    let plugin_path = parts.join("/") + ".py";
    
    let resolved_path = match utils::module::search_module(&plugin_path) {
        Some(path) => path,
        None => {
            println!("{} Plugin '{}' not found", "Error:".red().bold(), plugin_name);
            return;
        }
    };

    println!("Checking dependencies for: {}", plugin_name.cyan());
    println!();

    // Read the plugin file
    let code = match fs::read_to_string(&resolved_path) {
        Ok(code) => code,
        Err(e) => {
            println!("{} Failed to read plugin: {}", "Error:".red().bold(), e);
            return;
        }
    };

    // Extract dependencies from VALRADAR_CONFIG metadata or plugin class
    let dependencies = Python::with_gil(|py| -> Result<Vec<String>, String> {
        let module_name = parts.last().unwrap_or(&"unknown");
        
        let plugin_module = match PyModule::from_code(py, &code, &resolved_path, module_name) {
            Ok(p) => p,
            Err(e) => {
                return Err(format!("Failed to load plugin: {}", e.value(py)));
            }
        };

        let config = match plugin_module.getattr("VALRADAR_CONFIG") {
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

        // First, check for dependencies in metadata (legacy style)
        if let Ok(Some(metadata)) = config_dict.get_item("metadata") {
            if let Ok(metadata_dict) = metadata.extract::<&PyDict>() {
                if let Ok(Some(deps)) = metadata_dict.get_item("dependencies") {
                    if let Ok(deps_list) = deps.extract::<&PyList>() {
                        let mut result = Vec::new();
                        for item in deps_list {
                            if let Ok(s) = item.extract::<String>() {
                                result.push(s);
                            }
                        }
                        if !result.is_empty() {
                            return Ok(result);
                        }
                    }
                }
            }
        }

        // Second, check for 'plugin' instance with 'dependencies' attribute (SDK style)
        if let Ok(plugin_instance) = plugin_module.getattr("plugin") {
            if let Ok(deps) = plugin_instance.getattr("dependencies") {
                if let Ok(deps_list) = deps.extract::<&PyList>() {
                    let mut result = Vec::new();
                    for item in deps_list {
                        if let Ok(s) = item.extract::<String>() {
                            result.push(s);
                        }
                    }
                    return Ok(result);
                }
            }
        }

        Ok(Vec::new())
    });

    let deps = match dependencies {
        Ok(d) => d,
        Err(e) => {
            println!("{} {}", "Error:".red().bold(), e);
            return;
        }
    };

    if deps.is_empty() {
        println!("{}", "No dependencies declared for this plugin.".yellow());
        println!();
        println!("To add dependencies, include them in your plugin's metadata:");
        println!();
        println!("  class MyPlugin(Plugin):");
        println!("      dependencies = [\"requests\", \"beautifulsoup4\"]");
        return;
    }

    println!("Found {} dependenc{}:", deps.len(), if deps.len() == 1 { "y" } else { "ies" });
    for dep in &deps {
        println!("  - {}", dep);
    }
    println!();

    // Install with pip
    println!("{}", "Installing...".cyan());
    println!();

    let mut all_success = true;
    for dep in &deps {
        print!("  {} {} ... ", "→".blue(), dep);
        
        // Try pip first, then pip3
        let result = Command::new("pip")
            .args(["install", "--quiet", dep])
            .output()
            .or_else(|_| {
                Command::new("pip3")
                    .args(["install", "--quiet", dep])
                    .output()
            });

        match result {
            Ok(output) if output.status.success() => {
                println!("{}", "✓".green());
            }
            Ok(output) => {
                println!("{}", "✗".red());
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.is_empty() {
                    println!("    {}", stderr.trim());
                }
                all_success = false;
            }
            Err(e) => {
                println!("{}", "✗".red());
                println!("    pip not found: {}", e);
                all_success = false;
            }
        }
    }

    println!();
    if all_success {
        println!("{} All dependencies installed successfully!", "✓".green().bold());
    } else {
        println!("{} Some dependencies failed to install.", "⚠".yellow().bold());
        println!("Check the errors above and try installing manually with pip.");
    }
}
