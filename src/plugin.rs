use std::fs;
use std::path::Path;
use std::process::Command;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyString};
use pyo3::PyObject;

use crate::utils;

const IGNORE_KEYS: [&str; 3] = ["name", "description", "version"];

struct PluginMetadata<'a>(&'a PyDict);

impl<'a> PluginMetadata<'a> {
    fn new(metadata: &'a PyDict) -> Self {
        Self(metadata)
    }

    pub fn get_value(&self, key: &str) -> anyhow::Result<String> {
        let value = self.0.get_item(key)?;
        if let Some(value) = value {
            Ok(value.extract::<&PyString>()?.to_string())
        } else {
            Err(anyhow::anyhow!("Mandatory metadata key not found: '{}'", key))
        }
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
        let mut python_command =Command::new("python")
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
                return Err(anyhow::anyhow!("Failed to get python paths: neither python nor python3 is available"));
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

    fn create_interpreter(&self) -> anyhow::Result<(PyObject, PyObject)> {
        Python::with_gil(|py| {
            let plugin_name = Path::new(&self.name).file_name().unwrap();
            let sys = PyModule::import(py, "sys")?;
            let existing_paths = Self::get_existing_python_paths(py)?;
            utils::debug(&format!("Existing paths: {:?}", existing_paths));
            let _ = sys.setattr("path", existing_paths);
            utils::debug(&format!("Sys path: {:?}", sys.getattr("path")?));
            
            let plugin = PyModule::from_code(py, &self.code, &self.path, plugin_name.to_str().unwrap())?;
            let config = plugin.getattr("VALRADAR_CONFIG")?;
            
            Ok((plugin.into(), config.into()))
        })
    }

    pub fn get_metadata(&self) -> anyhow::Result<utils::PluginMetadata> {
        let (_plugin, config) = self.create_interpreter()?;
        let result = Python::with_gil(|py| {
            let config = match config.extract::<&PyDict>(py) {
                Ok(config) => config,
                Err(e) => {
                    utils::debug(&format!("Failed to get config: {}", e));
                    return Ok(utils::PluginMetadata::default());
                },
            };

            let metadata = match config.get_item("metadata") {
                Ok(metadata) => {
                    if let Some(metadata) = metadata {
                        metadata
                    } else {
                        return Ok(utils::PluginMetadata::default());
                    }
                },
                Err(e) => {
                    utils::debug(&format!("Failed to get metadata: {}", e));
                    return Ok(utils::PluginMetadata::default());
                },
            };

            let metadata = match metadata.extract::<&PyDict>() {
                Ok(metadata) => metadata,
                Err(e) => {
                    utils::debug(&format!("Failed to get metadata: {}", e));
                    return Ok(utils::PluginMetadata::default());
                },
            };

            let remaining = metadata.items().into_iter().map(|py_tuple| {
                let (key, value) = match py_tuple.extract::<(&PyString, &PyAny)>() {
                    Ok((key, value)) => (key.to_string(), value.to_string()),
                    Err(e) => {
                        utils::debug(&format!("Failed to get remaining: {}", e));
                        ("".to_string(), "".to_string())
                    },
                };
                (key, value)
            }).filter(|(key, _)| !IGNORE_KEYS.contains(&key.as_str())).collect();

            let plugin_metadata = PluginMetadata::new(&metadata);

            Ok(utils::PluginMetadata {
                name: plugin_metadata.get_value("name")?,
                version: plugin_metadata.get_value("version").unwrap_or("v0.0.0".to_string()),
                description: plugin_metadata.get_value("description")?,
                remaining,
            })
        });

        return result;
    }

    pub fn init(&self, args: &[String]) -> anyhow::Result<Vec<utils::ExecutionContext>> {
        let (_plugin, config) = self.create_interpreter()?;
        
        let result = Python::with_gil(|py| {
            let config = config.extract::<&PyDict>(py)?;
            let args_list = PyList::empty(py);
            for arg in args {
                args_list.append(arg).unwrap();
            }

            let init_func = match config.get_item("init") {
                Ok(init_func) => init_func.unwrap(),
                Err(e) => {
                    utils::debug(&format!("Failed to get init function: {}", e));
                    return Err(anyhow::anyhow!(e));
                },
            };

            let result = match init_func.call((args_list,), None) {
                Ok(value) => value,
                Err(e) => {
                    utils::debug(&format!("Failed to call init function: {}", e));
                    return Err(anyhow::anyhow!(e));
                },
            };

            utils::debug(&format!("Init function returned: {:?}", result));

            if let Ok(initial_data) = result.extract::<&PyList>() {
                Ok(initial_data.into_iter().map(|value| utils::ExecutionContext::new(value.into())).collect())
            } else {
                Err(anyhow::anyhow!("Init function execution failed"))
            }
        });

        return result;
    }

    pub fn collect_data(&self, data: &utils::ExecutionContext) -> anyhow::Result<Vec<utils::ExecutionContext>> {
        let (_, config) = self.create_interpreter()?;
        
        let result = Python::with_gil(|py| {
            let config = config.extract::<&PyDict>(py)?;
            let collect_data_func = match config.get_item("collect_data") {
                Ok(collect_data_func) => collect_data_func.unwrap(),
                Err(e) => {
                    utils::debug(&format!("Failed to get collect_data function: {}", e));
                    return Err(anyhow::anyhow!(e));
                },
            };
            
            let result = match collect_data_func.call((data.as_pyobject(),), None) {
                Ok(value) => value,
                Err(e) => {
                    utils::debug(&format!("Failed to call collect_data function: {}", e));
                    return Err(anyhow::anyhow!(e));
                },
            };

            if let Ok(collected_data) = result.extract::<&PyList>() {
                Ok(collected_data.into_iter().map(|value| utils::ExecutionContext::new(value.into())).collect())
            } else {
                Err(anyhow::anyhow!("Collect data function returned a non-list value"))
            }
        });

        return result;
    }
    
    pub fn process_data(&self, data: &utils::ExecutionContext) -> anyhow::Result<utils::ProcessingResult> {
        let (_, config) = self.create_interpreter()?;
        
        let result = Python::with_gil(|py| {
            let config = config.extract::<&PyDict>(py)?;
            let process_data_func = match config.get_item("process_data") {
                Ok(process_data_func) => process_data_func.unwrap(),
                Err(e) => {
                    utils::debug(&format!("Failed to get process_data function: {}", e));
                    return Err(anyhow::anyhow!(e));
                },
            };

            let result = match process_data_func.call((data.as_pyobject(),), None) {
                Ok(value) => value,
                Err(e) => {
                    utils::debug(&format!("Failed to call process_data function: {}", e));
                    return Err(anyhow::anyhow!(e));
                },
            };

            if let Ok(processed_data) = result.extract::<&PyDict>() {
                let keys = processed_data.keys().into_iter().map(|key| key.extract::<&PyString>().unwrap().to_string()).collect();
                let values = processed_data.values().into_iter().map(|value| value.extract::<&PyString>().unwrap().to_string()).collect();
                Ok(utils::ProcessingResult::new(keys, values))
            } else {
                Err(anyhow::anyhow!("Process data function returned a non-dict value"))
            }
        });

        return result;
    }
}

impl Default for Plugin {
    fn default() -> Self {
        Self::new("".to_string(), "".to_string())
    }
}
