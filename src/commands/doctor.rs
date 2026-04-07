use std::fs;
use colored::Colorize;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use crate::utils;

struct DiagnosticResult {
    check: String,
    passed: bool,
    message: String,
    suggestion: Option<String>,
}

pub fn doctor(plugin_name: &str) {
    // Find the plugin file
    let parts = plugin_name.split('.').collect::<Vec<&str>>();
    let plugin_path = parts.join("/") + ".py";
    
    let resolved_path = match utils::module::search_module(&plugin_path) {
        Some(path) => path,
        None => {
            println!("{} Plugin '{}' not found", "Error:".red().bold(), plugin_name);
            println!();
            println!("Make sure the plugin exists in one of:");
            println!("  - ./examples/");
            println!("  - ./modules/");
            println!("  - ~/.valradar/modules/");
            return;
        }
    };

    println!("{} {}", "Checking plugin:".bold(), plugin_name.cyan());
    println!("Path: {}", resolved_path);
    println!();

    let mut results = Vec::new();

    // Check 1: File is readable
    let code = match fs::read_to_string(&resolved_path) {
        Ok(code) => {
            results.push(DiagnosticResult {
                check: "File readable".to_string(),
                passed: true,
                message: "Plugin file can be read".to_string(),
                suggestion: None,
            });
            code
        }
        Err(e) => {
            results.push(DiagnosticResult {
                check: "File readable".to_string(),
                passed: false,
                message: format!("Cannot read file: {}", e),
                suggestion: Some("Check file permissions".to_string()),
            });
            print_results(&results);
            return;
        }
    };

    // Check 2: Valid Python syntax & can load
    let module_name = parts.last().unwrap_or(&"unknown");
    let load_result = Python::with_gil(|py| -> Result<(Py<PyAny>, Py<PyAny>), String> {
        let plugin = match PyModule::from_code(py, &code, &resolved_path, module_name) {
            Ok(p) => p,
            Err(e) => {
                return Err(format!("Python error: {}", e.value(py)));
            }
        };

        let config = match plugin.getattr("VALRADAR_CONFIG") {
            Ok(c) => c,
            Err(_) => {
                return Err("VALRADAR_CONFIG not defined".to_string());
            }
        };

        Ok((plugin.into(), config.into()))
    });

    let (plugin_obj, config_obj) = match load_result {
        Ok(objs) => {
            results.push(DiagnosticResult {
                check: "Python syntax".to_string(),
                passed: true,
                message: "Module loads without errors".to_string(),
                suggestion: None,
            });
            objs
        }
        Err(e) => {
            results.push(DiagnosticResult {
                check: "Python syntax".to_string(),
                passed: false,
                message: e.clone(),
                suggestion: if e.contains("VALRADAR_CONFIG") {
                    Some("Add VALRADAR_CONFIG = to_config(plugin) at the end of your file".to_string())
                } else {
                    Some("Fix the Python syntax error shown above".to_string())
                },
            });
            print_results(&results);
            return;
        }
    };

    // Check 3: VALRADAR_CONFIG exists (already verified above)
    results.push(DiagnosticResult {
        check: "VALRADAR_CONFIG exists".to_string(),
        passed: true,
        message: "Configuration dictionary found".to_string(),
        suggestion: None,
    });

    // Check 4: VALRADAR_CONFIG is a dict
    let config_dict_result = Python::with_gil(|py| -> Result<(), String> {
        match config_obj.extract::<&PyDict>(py) {
            Ok(_) => Ok(()),
            Err(_) => Err("VALRADAR_CONFIG is not a dictionary".to_string()),
        }
    });

    if let Err(e) = config_dict_result {
        results.push(DiagnosticResult {
            check: "Config is dict".to_string(),
            passed: false,
            message: e,
            suggestion: Some("Use to_config(plugin) to create the config dictionary".to_string()),
        });
        print_results(&results);
        return;
    }

    results.push(DiagnosticResult {
        check: "Config is dict".to_string(),
        passed: true,
        message: "VALRADAR_CONFIG is a valid dictionary".to_string(),
        suggestion: None,
    });

    // Check 5: Required keys exist and are callable
    let required_keys = ["init", "collect_data", "process_data"];
    
    Python::with_gil(|py| {
        let config = config_obj.extract::<&PyDict>(py).unwrap();
        
        for key in required_keys {
            match config.get_item(key) {
                Ok(Some(value)) => {
                    // Check if callable
                    if value.is_callable() {
                        results.push(DiagnosticResult {
                            check: format!("'{}' function", key),
                            passed: true,
                            message: format!("Function '{}' is defined and callable", key),
                            suggestion: None,
                        });
                    } else {
                        results.push(DiagnosticResult {
                            check: format!("'{}' function", key),
                            passed: false,
                            message: format!("'{}' exists but is not callable", key),
                            suggestion: Some(format!("Make sure '{}' is a function reference, not a string or other value", key)),
                        });
                    }
                }
                Ok(None) | Err(_) => {
                    results.push(DiagnosticResult {
                        check: format!("'{}' function", key),
                        passed: false,
                        message: format!("Required key '{}' is missing", key),
                        suggestion: Some(format!("Add '{}' to VALRADAR_CONFIG pointing to a function", key)),
                    });
                }
            }
        }

        // Check 6: Metadata (optional but recommended)
        match config.get_item("metadata") {
            Ok(Some(metadata)) => {
                if let Ok(meta_dict) = metadata.extract::<&PyDict>() {
                    let mut meta_issues = Vec::new();
                    
                    for key in ["name", "description", "version"] {
                        if meta_dict.get_item(key).ok().flatten().is_none() {
                            meta_issues.push(key);
                        }
                    }
                    
                    if meta_issues.is_empty() {
                        results.push(DiagnosticResult {
                            check: "Metadata".to_string(),
                            passed: true,
                            message: "All recommended metadata fields present".to_string(),
                            suggestion: None,
                        });
                    } else {
                        results.push(DiagnosticResult {
                            check: "Metadata".to_string(),
                            passed: true, // Still passes, just a warning
                            message: format!("Missing optional metadata: {}", meta_issues.join(", ")),
                            suggestion: Some("Add name, description, and version to your plugin class".to_string()),
                        });
                    }
                }
            }
            _ => {
                results.push(DiagnosticResult {
                    check: "Metadata".to_string(),
                    passed: true, // Still passes, just a warning
                    message: "No metadata defined (optional)".to_string(),
                    suggestion: Some("Add name, description, version attributes to your Plugin class".to_string()),
                });
            }
        }
    });

    // Also check if 'plugin' variable exists
    let _ = Python::with_gil(|py| -> Result<(), ()> {
        let plugin = plugin_obj.as_ref(py);
        match plugin.getattr("plugin") {
            Ok(_) => {
                results.push(DiagnosticResult {
                    check: "Plugin instance".to_string(),
                    passed: true,
                    message: "'plugin' instance exported".to_string(),
                    suggestion: None,
                });
            }
            Err(_) => {
                results.push(DiagnosticResult {
                    check: "Plugin instance".to_string(),
                    passed: true, // Not strictly required
                    message: "No 'plugin' instance exported (OK if using SDK)".to_string(),
                    suggestion: None,
                });
            }
        }
        Ok(())
    });

    print_results(&results);
}

fn print_results(results: &[DiagnosticResult]) {
    println!("{}", "Diagnostic Results".bold());
    println!("{}", "─".repeat(60));
    
    let mut failed_count = 0;
    
    for result in results {
        let status = if result.passed {
            "✓".green()
        } else {
            failed_count += 1;
            "✗".red()
        };
        
        println!("{} {} - {}", status, result.check.bold(), result.message);
        
        if !result.passed {
            if let Some(ref suggestion) = result.suggestion {
                println!("  {} {}", "→".blue(), suggestion);
            }
        }
    }
    
    println!("{}", "─".repeat(60));
    
    if failed_count == 0 {
        println!();
        println!("{} Plugin is valid and ready to use!", "✓".green().bold());
    } else {
        println!();
        println!("{} {} issue(s) found. Fix the errors above.", "✗".red().bold(), failed_count);
    }
}
