//! Command parsing for the interactive console

/// Represents a parsed command from user input
#[derive(Debug, Clone)]
pub enum Command {
    /// Load a module: use modules.web.emails
    Use(String),
    /// Show available modules: show modules
    ShowModules,
    /// Show current module options: show options
    ShowOptions,
    /// Show module info: info
    Info,
    /// Set an option value: set pattern emails=.*
    Set(String, String),
    /// Clear an option value: unset pattern
    Unset(String),
    /// Run the module: run https://example.com
    Run(Vec<String>),
    /// Unload current module: back
    Back,
    /// Show help: help or ?
    Help,
    /// Clear screen: clear
    Clear,
    /// Exit the console: exit or quit
    Exit,
    /// Empty input (just pressed enter)
    Empty,
    /// Unknown command
    Unknown(String),
}

/// Parse a line of input into a Command
pub fn parse_command(input: &str) -> Command {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Command::Empty;
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    match parts.as_slice() {
        ["use", module] => Command::Use(module.to_string()),
        ["use"] => Command::Unknown("use requires a module name".to_string()),

        ["show", "modules"] => Command::ShowModules,
        ["show", "options"] => Command::ShowOptions,
        ["show"] => Command::Unknown("show requires 'modules' or 'options'".to_string()),

        ["info"] => Command::Info,

        ["set", opt, value @ ..] if !value.is_empty() => {
            Command::Set(opt.to_string(), value.join(" "))
        }
        ["set", _] => Command::Unknown("set requires option and value".to_string()),
        ["set"] => Command::Unknown("set requires option and value".to_string()),

        ["unset", opt] => Command::Unset(opt.to_string()),
        ["unset"] => Command::Unknown("unset requires option name".to_string()),

        ["run", targets @ ..] => Command::Run(targets.iter().map(|s| s.to_string()).collect()),

        ["back"] => Command::Back,

        ["help"] | ["?"] => Command::Help,

        ["clear"] => Command::Clear,

        ["exit"] | ["quit"] => Command::Exit,

        _ => Command::Unknown(format!("Unknown command: {}", parts[0])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_use() {
        match parse_command("use modules.web.emails") {
            Command::Use(m) => assert_eq!(m, "modules.web.emails"),
            _ => panic!("Expected Use command"),
        }
    }

    #[test]
    fn test_parse_set() {
        match parse_command("set pattern emails=[a-z]+@[a-z]+") {
            Command::Set(opt, val) => {
                assert_eq!(opt, "pattern");
                assert_eq!(val, "emails=[a-z]+@[a-z]+");
            }
            _ => panic!("Expected Set command"),
        }
    }

    #[test]
    fn test_parse_run() {
        match parse_command("run https://a.com https://b.com") {
            Command::Run(targets) => {
                assert_eq!(targets.len(), 2);
                assert_eq!(targets[0], "https://a.com");
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_parse_empty() {
        match parse_command("") {
            Command::Empty => {}
            _ => panic!("Expected Empty command"),
        }
    }
}
