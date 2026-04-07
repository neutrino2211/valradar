use std::fs;
use std::path::Path;
use colored::Colorize;

const PLUGIN_TEMPLATE: &str = r#""""{{name}} - A Valradar plugin.

TODO: Add description of what this plugin does.

Usage:
    valradar {{module_path}} <arg1> [arg2...]
"""

from typing import Any

from valradar import Context, Plugin, to_config


class {{class_name}}Plugin(Plugin):
    """TODO: Describe what this plugin does."""

    name = "{{name}}"
    description = "TODO: Add description"
    version = "0.1.0"
    author = "TODO: Your name"
    tags = ["TODO"]  # e.g., ["web", "scraping", "recon"]
    
    # Optional: List pip dependencies here
    # dependencies = ["requests", "beautifulsoup4"]

    def init(self, args: list[str]) -> list[Context]:
        """Initialize the plugin with command line arguments.
        
        Args:
            args: Command line arguments passed after the plugin name
            
        Returns:
            List of Context objects to process
        """
        if not args:
            print("Usage: valradar {{module_path}} <arg1> [arg2...]")
            exit(1)

        # TODO: Parse args and create initial contexts
        # Example: return [Context(url=url) for url in args]
        return [Context(data=arg) for arg in args]

    def collect(self, ctx: Context) -> list[Context]:
        """Collect data from a context.
        
        This is called for each context from init() and any new contexts
        returned by previous collect() calls (up to the configured depth).
        
        Args:
            ctx: The context to process
            
        Returns:
            List of new Context objects for further processing
        """
        # TODO: Implement data collection logic
        # Example: Fetch URL, extract data, return new contexts to follow
        
        data = ctx.get("data", "")
        
        # Store collected data in context for process()
        ctx.set("result", f"Collected: {data}")
        
        # Return empty list if no recursive collection needed
        return []

    def process(self, ctx: Context) -> dict[str, Any] | None:
        """Process collected data and return results.
        
        Called for each context after collection is complete.
        
        Args:
            ctx: The context with collected data
            
        Returns:
            Dict of key-value pairs to display, or None to skip
        """
        result = ctx.get("result")
        
        if result:
            return {
                "input": ctx.get("data", "")[:80],
                "output": result,
            }
        
        return None


# Instantiate and export for Valradar runtime
plugin = {{class_name}}Plugin()
VALRADAR_CONFIG = to_config(plugin)
"#;

fn to_class_name(name: &str) -> String {
    // Convert kebab-case or snake_case to PascalCase
    name.split(|c| c == '-' || c == '_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

pub fn new_plugin(name: &str, path: Option<&str>) {
    let class_name = to_class_name(name);
    
    // Generate the plugin code from template
    let code = PLUGIN_TEMPLATE
        .replace("{{name}}", name)
        .replace("{{class_name}}", &class_name)
        .replace("{{module_path}}", &format!("<module>.{}", name));
    
    // Determine output path
    let filename = format!("{}.py", name);
    let output_path = match path {
        Some(p) => {
            let p = Path::new(p);
            if p.is_dir() {
                p.join(&filename)
            } else {
                p.to_path_buf()
            }
        }
        None => Path::new(&filename).to_path_buf(),
    };
    
    // Check if file already exists
    if output_path.exists() {
        println!("{} File already exists: {}", "Error:".red().bold(), output_path.display());
        println!("Use a different name or delete the existing file first.");
        return;
    }
    
    // Write the file
    match fs::write(&output_path, code) {
        Ok(_) => {
            println!("{} Created plugin: {}", "✓".green().bold(), output_path.display());
            println!();
            println!("Next steps:");
            println!("  1. Edit {} and implement your logic", output_path.display());
            println!("  2. Update the metadata (name, description, author, tags)");
            println!("  3. Test with: valradar {} <args>", name);
            println!();
            println!("See examples/ for reference implementations.");
        }
        Err(e) => {
            println!("{} Failed to create plugin: {}", "Error:".red().bold(), e);
        }
    }
}
