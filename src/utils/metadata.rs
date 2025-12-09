use std::fmt;

/// Metadata for a plugin
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub remaining: Vec<(String, String)>,
}

impl Default for PluginMetadata {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            version: "".to_string(),
            description: "".to_string(),
            remaining: vec![],
        }
    }
}

impl fmt::Display for PluginMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let remaining = self
            .remaining
            .iter()
            .map(|(key, value)| format!("{}: {}", key, value))
            .collect::<Vec<String>>()
            .join("\n");

        let folded_description = self.description.clone();

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
