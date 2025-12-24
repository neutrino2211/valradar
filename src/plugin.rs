use crate::utils::{self, PluginInstance, ProcessingResult, TaskRequest, YieldValue};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyIterator, PyList};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Metadata about a plugin module
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub options: Vec<OptionInfo>,
}

#[derive(Debug, Clone)]
pub struct OptionInfo {
    pub name: String,
    pub opt_type: String,
    pub default: Option<String>,
    pub required: bool,
    pub help: String,
}

impl Default for PluginInfo {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            description: String::new(),
            author: String::new(),
            version: "0.1.0".to_string(),
            options: vec![],
        }
    }
}

impl std::fmt::Display for PluginInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name: {}", self.name)?;
        writeln!(f, "Version: {}", self.version)?;
        if !self.description.is_empty() {
            writeln!(f, "Description: {}", self.description)?;
        }
        if !self.author.is_empty() {
            writeln!(f, "Author: {}", self.author)?;
        }
        if !self.options.is_empty() {
            writeln!(f, "Options:")?;
            for opt in &self.options {
                let req = if opt.required { " (required)" } else { "" };
                let def = opt
                    .default
                    .as_ref()
                    .map(|d| format!(" [default: {}]", d))
                    .unwrap_or_default();
                writeln!(f, "  --{}: {}{}{}", opt.name, opt.opt_type, req, def)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct Plugin {
    pub name: String,
    pub path: String,
    code: String,
}

impl Plugin {
    fn get_existing_python_paths(py: Python<'_>) -> anyhow::Result<&PyList> {
        let mut python_command = Command::new("python")
            .arg("-c")
            .arg("import sys; print(sys.path)")
            .output();

        let output_string;

        if let Ok(output) = python_command {
            output_string = String::from_utf8(output.stdout).unwrap();
        } else {
            python_command = Command::new("python3")
                .arg("-c")
                .arg("import sys; print(sys.path)")
                .output();

            if let Ok(output) = python_command {
                output_string = String::from_utf8(output.stdout).unwrap();
            } else {
                return Err(anyhow::anyhow!(
                    "Failed to get python paths: neither python nor python3 is available"
                ));
            }
        }

        match Python::eval(py, output_string.as_str(), None, None) {
            Ok(path_list) => Ok(path_list.extract::<&PyList>()?),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    }

    pub fn new(name: String, path: String) -> Self {
        let code = fs::read_to_string(&path).unwrap_or_else(|_| String::new());
        Self { name, path, code }
    }

    /// Load the Python module and return the MODULE_CLASS
    fn load_module<'py>(&self, py: Python<'py>) -> anyhow::Result<&'py PyAny> {
        let plugin_name = Path::new(&self.name).file_name().unwrap();
        let sys = PyModule::import(py, "sys")?;
        let existing_paths = Self::get_existing_python_paths(py)?;

        // Add the project root to Python path for valradar SDK imports
        let project_root = Path::new(&self.path)
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(Path::new("."));

        let new_paths = PyList::new(py, existing_paths.iter());
        new_paths.insert(0, project_root.to_string_lossy().to_string())?;

        // Also add current working directory
        if let Ok(cwd) = std::env::current_dir() {
            new_paths.insert(0, cwd.to_string_lossy().to_string())?;
        }

        utils::debug(&format!("Python path includes: {:?}", project_root));
        let _ = sys.setattr("path", new_paths);

        let plugin =
            PyModule::from_code(py, &self.code, &self.path, plugin_name.to_str().unwrap())?;
        let module_class = plugin.getattr("MODULE_CLASS")?;

        Ok(module_class)
    }

    /// Get plugin metadata without instantiating
    pub fn get_info(&self) -> anyhow::Result<PluginInfo> {
        Python::with_gil(|py| {
            let module_class = self.load_module(py)?;
            utils::debug(&format!("Loaded module class: {:?}", module_class));

            let name = module_class
                .getattr("name")
                .and_then(|v| v.extract::<String>())
                .unwrap_or_else(|_| "Unknown".to_string());

            let description = module_class
                .getattr("description")
                .and_then(|v| v.extract::<String>())
                .unwrap_or_default();

            let author = module_class
                .getattr("author")
                .and_then(|v| v.extract::<String>())
                .unwrap_or_default();

            let version = module_class
                .getattr("version")
                .and_then(|v| v.extract::<String>())
                .unwrap_or_else(|_| "0.1.0".to_string());

            let mut options = vec![];
            if let Ok(opts_list) = module_class.getattr("options") {
                if let Ok(opts) = opts_list.extract::<&PyList>() {
                    for opt in opts.iter() {
                        let opt_name = opt
                            .getattr("name")
                            .and_then(|v| v.extract::<String>())
                            .unwrap_or_default();
                        let opt_type = opt
                            .getattr("type")
                            .and_then(|v| v.extract::<String>())
                            .unwrap_or_else(|_| "str".to_string());
                        let default = opt.getattr("default").ok().and_then(|v| {
                            if v.is_none() {
                                None
                            } else {
                                v.extract::<String>().ok()
                            }
                        });
                        let required = opt
                            .getattr("required")
                            .and_then(|v| v.extract::<bool>())
                            .unwrap_or(false);
                        let help = opt
                            .getattr("help")
                            .and_then(|v| v.extract::<String>())
                            .unwrap_or_default();

                        options.push(OptionInfo {
                            name: opt_name,
                            opt_type,
                            default,
                            required,
                            help,
                        });
                    }
                }
            }

            Ok(PluginInfo {
                name,
                description,
                author,
                version,
                options,
            })
        })
    }

    /// For backward compatibility with metadata display
    pub fn get_metadata(&self) -> anyhow::Result<utils::PluginMetadata> {
        let info = self.get_info()?;
        Ok(utils::PluginMetadata {
            name: info.name,
            description: info.description,
            version: info.version,
            options: info.options,
            remaining: vec![("author".to_string(), info.author)],
        })
    }

    /// Instantiate the plugin class and call setup()
    pub fn instantiate(&self) -> anyhow::Result<PluginInstance> {
        Python::with_gil(|py| {
            let module_class = self.load_module(py)?;

            // Instantiate: instance = MODULE_CLASS()
            let instance = module_class.call0()?;
            utils::debug(&format!("Instantiated module: {:?}", instance));

            // Call setup() if it exists
            if instance.hasattr("setup")? {
                instance.call_method0("setup")?;
                utils::debug("Called setup() on module instance");
            }

            Ok(PluginInstance::new(instance.into(), self.name.clone()))
        })
    }

    /// Run the generator for a target and collect all yielded values
    pub fn run_target(
        &self,
        instance: &PluginInstance,
        target: &str,
        kwargs: &HashMap<String, String>,
    ) -> anyhow::Result<Vec<YieldValue>> {
        Python::with_gil(|py| {
            // Build kwargs dict for Python
            let py_kwargs = PyDict::new(py);
            for (key, value) in kwargs {
                py_kwargs.set_item(key, value)?;
            }

            // Call run(target, **kwargs) which returns a generator
            let generator = instance
                .py_instance
                .call_method(py, "run", (target,), Some(py_kwargs))?;

            let mut yields = vec![];

            // Import SDK types for isinstance checks
            let sdk = PyModule::import(py, "valradar.sdk")?;
            let result_class = sdk.getattr("Result")?;
            let task_class = sdk.getattr("Task")?;

            // Iterate the generator
            let iterator = PyIterator::from_object(generator.as_ref(py))?;

            for item in iterator {
                let item = item?;

                if item.is_instance(result_class)? {
                    // Extract Result fields
                    let host = item
                        .getattr("host")
                        .and_then(|v| v.extract::<String>())
                        .unwrap_or_default();

                    let data = item.getattr("data")?;
                    let data_dict = data.extract::<&PyDict>()?;

                    let mut data_map: HashMap<String, String> = HashMap::new();
                    for (key, value) in data_dict.iter() {
                        let k = key.extract::<String>()?;
                        let v = value.str()?.to_string();
                        data_map.insert(k, v);
                    }

                    let result = ProcessingResult::from_data(data_map, host);
                    yields.push(YieldValue::Result(result));
                } else if item.is_instance(task_class)? {
                    // Extract Task fields
                    let target = item.getattr("target")?.extract::<String>()?;

                    let kwargs_dict = item.getattr("kwargs")?;
                    let mut kwargs: HashMap<String, String> = HashMap::new();
                    if let Ok(dict) = kwargs_dict.extract::<&PyDict>() {
                        for (key, value) in dict.iter() {
                            let k = key.extract::<String>()?;
                            let v = value.str()?.to_string();
                            kwargs.insert(k, v);
                        }
                    }

                    let task = TaskRequest::with_kwargs(target, kwargs);
                    yields.push(YieldValue::Task(task));
                } else {
                    utils::debug(&format!("Unknown yielded type: {:?}", item));
                }
            }

            Ok(yields)
        })
    }
}

impl Default for Plugin {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}
