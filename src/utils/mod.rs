use std::{env, fmt};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use comfy_table::Color;
use pyo3::prelude::*;
use terminal_size::{terminal_size, Height, Width};
use colored::Colorize;

pub fn debug(message: &str) -> () {
    if env::var("VALRADAR_DEBUG").is_ok() {
        println!("{}", message);
    }
}

pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub remaining: Vec<(String, String)>,
}

impl Default for PluginMetadata {
    fn default() -> Self {
        Self { name: "".to_string(), version: "".to_string(), description: "".to_string(), remaining: vec![] }
    }
}

impl fmt::Display for PluginMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let remaining = self.remaining.iter().map(|(key, value)| format!("{}: {}", key, value)).collect::<Vec<String>>().join("\n");
        let folded_description: String;

        if self.description.len() > 40 {
            folded_description = self.description.chars().take(40).collect::<Vec<char>>().iter().map(|c| c.to_string()).collect::<Vec<String>>().join("\n").into();
        } else {
            folded_description = self.description.clone();
        }

        write!(
            f,
            "{} v{}\n{}\n{}\n{}",
            self.name,
            self.version,
            folded_description,
            "=".repeat(folded_description.len()),
            remaining
        )
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionContext(PyObject);

impl ExecutionContext {
    pub fn new(obj: PyObject) -> Self {
        Self(obj)
    }

    pub fn as_pyobject(&self) -> &PyObject {
        &self.0
    }
}

impl fmt::Display for ExecutionContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExecutionContext {{ {} }}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub keys: Vec<String>,
    pub values: Vec<String>,
}

impl ProcessingResult {
    pub fn new(keys: Vec<String>, values: Vec<String>) -> Self {
        Self { keys, values }
    }
}

pub struct ProcessedData(pub Vec<ProcessingResult>);

impl ProcessedData {
    pub fn new(results: Vec<ProcessingResult>) -> Self {
        Self(results)
    }
}

impl fmt::Display for ProcessedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return write!(f, "No data to display");
        }
        let mut width = 80;
        if let Some((Width(terminal_width), Height(_))) = terminal_size() {
            width = terminal_width;
        }
        let mut table = Table::new();
        let headers = self.0[0].keys
            .clone()
            .into_iter()
            .map(|key| 
                Cell::new(key.to_uppercase().as_str())
                .fg(Color::Green)
            )
            .collect::<Vec<Cell>>();

        table.load_preset(UTF8_FULL);
        table.apply_modifier(UTF8_ROUND_CORNERS);
        table.set_header(headers);
        table.set_width(width);
        for result in self.0.clone() {
            let row = result.values
                .clone()
                .into_iter()
                .map(|value|
                    Cell::new(value.as_str())
                    .fg(Color::Blue)
                )
                .collect::<Vec<Cell>>();
            table.add_row(row);
        }
        write!(f, "{}", table)
    }
}

impl fmt::Display for ProcessingResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProcessingResult {{ keys: {:?}, values: {:?} }}", self.keys, self.values)
    }
}

pub fn print_banner(plugin_metadata: &PluginMetadata) {
    let raw_banner = format!("
888     888     d8888 888      8888888b.         d8888 8888888b.        d8888 8888888b.  
888     888    d88888 888      888   Y88b       d88888 888  'Y88b      d88888 888   Y88b 
888     888   d88P888 888      888    888      d88P888 888    888     d88P888 888    888 
Y88b   d88P  d88P 888 888      888   d88P     d88P 888 888    888    d88P 888 888   d88P 
 Y88b d88P  d88P  888 888      8888888P'     d88P  888 888    888   d88P  888 8888888P'  
  Y88o88P  d88P   888 888      888 T88b     d88P   888 888    888  d88P   888 888 T88b   
   Y888P  d8888888888 888      888  T88b   d8888888888 888  .d88P d8888888888 888  T88b  
    Y8P  d88P     888 88888888 888   T88b d88P     888 8888888P' d88P     888 888   T88b 
                                                                                         
{} v{}                                                                                    
{}                                                                                         
    ", plugin_metadata.name.bold().white(), plugin_metadata.version.bold().yellow(), plugin_metadata.description.bold().blue());
    println!("{}", raw_banner.blue());
}
