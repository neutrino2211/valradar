use std::collections::HashMap;
use std::fmt;
use pyo3::prelude::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use comfy_table::Color;
use comfy_table::CellAlignment;
use terminal_size::{terminal_size, Height, Width};

/// A task request yielded by Python to scan a new target
#[derive(Debug, Clone)]
pub struct TaskRequest {
    pub target: String,
    pub kwargs: HashMap<String, String>,
}

impl TaskRequest {
    pub fn new(target: String) -> Self {
        Self {
            target,
            kwargs: HashMap::new(),
        }
    }

    pub fn with_kwargs(target: String, kwargs: HashMap<String, String>) -> Self {
        Self { target, kwargs }
    }
}

/// Discriminated union for values yielded by Python generators
#[derive(Debug, Clone)]
pub enum YieldValue {
    /// A finding/result to be collected
    Result(ProcessingResult),
    /// A new task to be queued
    Task(TaskRequest),
}

/// Holds a Python module instance for repeated method calls
pub struct PluginInstance {
    pub py_instance: PyObject,
    pub module_name: String,
}

impl PluginInstance {
    pub fn new(py_instance: PyObject, module_name: String) -> Self {
        Self {
            py_instance,
            module_name,
        }
    }
}

// PluginInstance is Send because PyObject handles GIL internally
unsafe impl Send for PluginInstance {}
unsafe impl Sync for PluginInstance {}

/// Result of processing data - a row in the output table
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub keys: Vec<String>,
    pub values: Vec<String>,
    pub host: String,
}

impl ProcessingResult {
    pub fn new(keys: Vec<String>, values: Vec<String>) -> Self {
        Self {
            keys,
            values,
            host: String::new(),
        }
    }

    pub fn with_host(keys: Vec<String>, values: Vec<String>, host: String) -> Self {
        Self { keys, values, host }
    }

    pub fn from_data(data: HashMap<String, String>, host: String) -> Self {
        let keys: Vec<String> = data.keys().cloned().collect();
        let values: Vec<String> = keys.iter().map(|k| data.get(k).cloned().unwrap_or_default()).collect();
        Self { keys, values, host }
    }
}

impl fmt::Display for ProcessingResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProcessingResult {{ host: {}, keys: {:?}, values: {:?} }}", self.host, self.keys, self.values)
    }
}

/// Container for processed data - renders as a table
pub struct ProcessedData(pub Vec<ProcessingResult>);

impl ProcessedData {
    pub fn new(results: Vec<ProcessingResult>) -> Self {
        Self(results)
    }
}

impl fmt::Display for ProcessedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return write!(f, "No results found");
        }

        let mut width = 80;
        if let Some((Width(terminal_width), Height(_))) = terminal_size() {
            width = terminal_width;
        }

        let mut table = Table::new();

        // Build headers from first result, prepending "host" if present
        let mut headers: Vec<Cell> = vec![];
        let first = &self.0[0];

        if !first.host.is_empty() {
            headers.push(
                Cell::new("HOST")
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Green)
            );
        }

        for key in &first.keys {
            headers.push(
                Cell::new(key.to_uppercase().as_str())
                    .set_alignment(CellAlignment::Center)
                    .fg(Color::Green)
            );
        }

        table.load_preset(UTF8_FULL);
        table.apply_modifier(UTF8_ROUND_CORNERS);
        table.set_header(headers.clone());

        let header_len: u32 = headers.len().max(1) as u32;
        for idx in 0..headers.len() {
            if let Some(column) = table.column_mut(idx) {
                column.set_constraint(ColumnConstraint::UpperBoundary(
                    comfy_table::Width::Fixed((width as u32 / header_len).try_into().unwrap())
                ));
            }
        }

        table.set_width(width);

        for result in &self.0 {
            let mut row: Vec<Cell> = vec![];

            if !result.host.is_empty() {
                row.push(Cell::new(&result.host).fg(Color::Cyan));
            }

            for value in &result.values {
                row.push(Cell::new(value.as_str()).fg(Color::Blue));
            }

            table.add_row(row);
        }

        write!(f, "{}", table)
    }
}
