// Re-export all utility modules
pub mod metadata;
pub mod context;
pub mod display;
pub mod logging;
pub mod module;
pub mod license;

// Re-export commonly used items for convenience
pub use metadata::PluginMetadata;
pub use context::{ProcessingResult, ProcessedData, TaskRequest, YieldValue, PluginInstance};
pub use logging::debug;
pub use display::print_banner;
pub use module::{list_modules, search_module, ModuleEntry};
