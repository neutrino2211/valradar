// Re-export all utility modules
pub mod metadata;
pub mod context;
pub mod display;
pub mod logging;
pub mod module;
pub mod license;
pub mod errors;

// Re-export commonly used items for convenience
pub use metadata::PluginMetadata;
pub use context::{ExecutionContext, ProcessingResult, ProcessedData};
pub use logging::debug;
pub use display::print_banner;
pub use errors::{format_python_error, get_python_type_name, PluginValidationError};
