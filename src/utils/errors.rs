use pyo3::prelude::*;
use pyo3::types::PyTraceback;

/// Formats a Python error with full traceback for clear error reporting
pub fn format_python_error(py: Python<'_>, err: &PyErr) -> String {
    let mut result = String::new();
    
    // Get the error type and message
    let err_type = err.get_type(py).name().unwrap_or("UnknownError");
    let err_value = err.value(py).to_string();
    
    result.push_str(&format!("\n╭─ Python Error ─────────────────────────────────────────\n"));
    result.push_str(&format!("│ {}: {}\n", err_type, err_value));
    
    // Extract and format traceback if available
    if let Some(traceback) = err.traceback(py) {
        if let Ok(tb_str) = format_traceback(traceback) {
            result.push_str("│\n│ Traceback:\n");
            for line in tb_str.lines() {
                result.push_str(&format!("│   {}\n", line));
            }
        }
    }
    
    result.push_str("╰────────────────────────────────────────────────────────\n");
    result
}

/// Format a Python traceback object into a string
fn format_traceback(traceback: &PyTraceback) -> PyResult<String> {
    traceback.format()
}

/// Error types for plugin validation
#[derive(Debug)]
pub enum PluginValidationError {
    MissingConfig,
    MissingConfigKey(String),
    WrongReturnType { function: String, expected: String, got: String },
    FunctionCallFailed { function: String, error: String },
    ConfigNotDict,
}

impl std::fmt::Display for PluginValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginValidationError::MissingConfig => {
                write!(f, "╭─ Plugin Error ─────────────────────────────────────────\n")?;
                write!(f, "│ Missing VALRADAR_CONFIG\n")?;
                write!(f, "│\n")?;
                write!(f, "│ Your plugin must define a VALRADAR_CONFIG dictionary.\n")?;
                write!(f, "│\n")?;
                write!(f, "│ Example:\n")?;
                write!(f, "│   VALRADAR_CONFIG = {{\n")?;
                write!(f, "│       \"metadata\": {{\n")?;
                write!(f, "│           \"name\": \"my-plugin\",\n")?;
                write!(f, "│           \"version\": \"1.0.0\",\n")?;
                write!(f, "│           \"description\": \"What it does\"\n")?;
                write!(f, "│       }},\n")?;
                write!(f, "│       \"init\": init_function,\n")?;
                write!(f, "│       \"collect_data\": collect_function,\n")?;
                write!(f, "│       \"process_data\": process_function\n")?;
                write!(f, "│   }}\n")?;
                write!(f, "╰────────────────────────────────────────────────────────")
            }
            PluginValidationError::MissingConfigKey(key) => {
                write!(f, "╭─ Plugin Error ─────────────────────────────────────────\n")?;
                write!(f, "│ Missing required key in VALRADAR_CONFIG: '{}'\n", key)?;
                write!(f, "│\n")?;
                write!(f, "│ VALRADAR_CONFIG must contain these keys:\n")?;
                write!(f, "│   - init: function(args: list) -> list\n")?;
                write!(f, "│   - collect_data: function(ctx) -> list\n")?;
                write!(f, "│   - process_data: function(ctx) -> dict\n")?;
                write!(f, "│\n")?;
                write!(f, "│ Example:\n")?;
                write!(f, "│   def {}(arg):\n", key)?;
                write!(f, "│       # Your implementation\n")?;
                write!(f, "│       pass\n")?;
                write!(f, "│\n")?;
                write!(f, "│   VALRADAR_CONFIG = {{\n")?;
                write!(f, "│       \"{}\": {},\n", key, key)?;
                write!(f, "│       # ... other keys\n")?;
                write!(f, "│   }}\n")?;
                write!(f, "╰────────────────────────────────────────────────────────")
            }
            PluginValidationError::WrongReturnType { function, expected, got } => {
                write!(f, "╭─ Plugin Error ─────────────────────────────────────────\n")?;
                write!(f, "│ Wrong return type from '{}'\n", function)?;
                write!(f, "│\n")?;
                write!(f, "│ Expected: {}\n", expected)?;
                write!(f, "│ Got:      {}\n", got)?;
                write!(f, "│\n")?;
                match function.as_str() {
                    "init" => {
                        write!(f, "│ The init function must return a list of initial contexts.\n")?;
                        write!(f, "│\n")?;
                        write!(f, "│ Example:\n")?;
                        write!(f, "│   def init(args):\n")?;
                        write!(f, "│       return [\"item1\", \"item2\"]  # Must be a list!\n")?;
                    }
                    "collect_data" => {
                        write!(f, "│ The collect_data function must return a list.\n")?;
                        write!(f, "│\n")?;
                        write!(f, "│ Example:\n")?;
                        write!(f, "│   def collect_data(ctx):\n")?;
                        write!(f, "│       return [data1, data2]  # Must be a list!\n")?;
                    }
                    "process_data" => {
                        write!(f, "│ The process_data function must return a dict.\n")?;
                        write!(f, "│\n")?;
                        write!(f, "│ Example:\n")?;
                        write!(f, "│   def process_data(ctx):\n")?;
                        write!(f, "│       return {{\"key\": \"value\"}}  # Must be a dict!\n")?;
                    }
                    _ => {}
                }
                write!(f, "╰────────────────────────────────────────────────────────")
            }
            PluginValidationError::FunctionCallFailed { function, error } => {
                write!(f, "╭─ Plugin Error ─────────────────────────────────────────\n")?;
                write!(f, "│ Error calling '{}'\n", function)?;
                write!(f, "│\n")?;
                write!(f, "│ {}\n", error)?;
                write!(f, "╰────────────────────────────────────────────────────────")
            }
            PluginValidationError::ConfigNotDict => {
                write!(f, "╭─ Plugin Error ─────────────────────────────────────────\n")?;
                write!(f, "│ VALRADAR_CONFIG must be a dictionary\n")?;
                write!(f, "│\n")?;
                write!(f, "│ Make sure VALRADAR_CONFIG is defined as a dict, not\n")?;
                write!(f, "│ another type like a string or list.\n")?;
                write!(f, "│\n")?;
                write!(f, "│ Correct:\n")?;
                write!(f, "│   VALRADAR_CONFIG = {{\n")?;
                write!(f, "│       \"init\": init_function,\n")?;
                write!(f, "│       ...\n")?;
                write!(f, "│   }}\n")?;
                write!(f, "╰────────────────────────────────────────────────────────")
            }
        }
    }
}

impl std::error::Error for PluginValidationError {}

/// Get the Python type name for display
pub fn get_python_type_name(_py: Python<'_>, obj: &PyAny) -> String {
    obj.get_type().name().unwrap_or("unknown").to_string()
}
