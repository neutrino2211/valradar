use std::fmt;
use pyo3::prelude::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use comfy_table::Color;
use comfy_table::CellAlignment;
use terminal_size::{terminal_size, Height, Width};

/// Execution context for a plugin
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

/// Result of processing data
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

impl fmt::Display for ProcessingResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProcessingResult {{ keys: {:?}, values: {:?} }}", self.keys, self.values)
    }
}

/// Container for processed data
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
                .set_alignment(CellAlignment::Center)
                .fg(Color::Green)
            )
            .collect::<Vec<Cell>>();

        table.load_preset(UTF8_FULL);
        table.apply_modifier(UTF8_ROUND_CORNERS);
        table.set_header(headers.clone());
        
        let header_len: u32 = headers.len() as u32;
        for idx in 0..headers.len() {
            if let Some(column) = table.column_mut(idx) {
                column.set_constraint(ColumnConstraint::UpperBoundary(comfy_table::Width::Fixed((width as u32 / header_len).try_into().unwrap())));
            }
        }
        
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