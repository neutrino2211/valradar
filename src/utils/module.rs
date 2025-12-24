use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use homedir::my_home;

/// Entry representing a discovered module
#[derive(Debug, Clone)]
pub struct ModuleEntry {
    pub name: String,
    pub path: PathBuf,
}

/// List all available Python modules in the modules directories
pub fn list_modules() -> Vec<ModuleEntry> {
    let mut modules = Vec::new();
    let current_dir = env::current_dir().unwrap_or_default();
    let home_dir = my_home().ok().flatten().unwrap_or_default();

    let search_paths = [
        current_dir.join("modules"),
        home_dir.join(".valradar").join("modules"),
    ];

    for base_path in search_paths {
        if base_path.exists() {
            walk_modules(&base_path, &base_path, &mut modules);
        }
    }

    // Sort by name for consistent ordering
    modules.sort_by(|a, b| a.name.cmp(&b.name));
    modules
}

fn walk_modules(base: &Path, current: &Path, modules: &mut Vec<ModuleEntry>) {
    if let Ok(entries) = fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip __pycache__ and hidden directories
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if !name_str.starts_with('.') && name_str != "__pycache__" {
                        walk_modules(base, &path, modules);
                    }
                }
            } else if path.extension().map(|e| e == "py").unwrap_or(false) {
                // Skip __init__.py files
                if let Some(name) = path.file_name() {
                    if name.to_string_lossy() == "__init__.py" {
                        continue;
                    }
                }
                // Convert path to module name (e.g., modules/web/emails.py -> modules.web.emails)
                if let Ok(rel) = path.strip_prefix(base) {
                    let name = format!(
                        "modules.{}",
                        rel.with_extension("")
                            .to_string_lossy()
                            .replace(std::path::MAIN_SEPARATOR, ".")
                    );
                    modules.push(ModuleEntry { name, path });
                }
            }
        }
    }
}

pub fn search_module(module_name: &str) -> Option<String> {
    let current_dir = env::current_dir().unwrap();
    let home_dir = match my_home() {
        Ok(dir) => dir.unwrap(),
        Err(_) => return None,
    };

    let mut module_path = current_dir.join(module_name);

    if module_path.exists() {
        return Some(module_path.to_string_lossy().to_string());
    }

    module_path = home_dir.join(".valradar").join("modules").join(module_name);

    if module_path.exists() {
        return Some(module_path.to_string_lossy().to_string());
    }

    None
}