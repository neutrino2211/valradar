//! Interactive console for Valradar (Metasploit-style TUI)

use std::borrow::Cow;
use std::collections::HashMap;
use std::time::Instant;

use anyhow::Result;
use crossterm::{execute, terminal};
use homedir::my_home;
use indicatif::{ProgressBar, ProgressStyle};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};

use crate::commands::{parse_command, Command};
use crate::plugin::Plugin;
use crate::utils::module::{list_modules, search_module, ModuleEntry};
use crate::utils::{debug, PluginInstance};
use crate::Orchestrator;

/// Interactive session context
struct InteractiveContext {
    current_module: Option<Plugin>,
    current_instance: Option<PluginInstance>,
    module_name: Option<String>,
    options: HashMap<String, String>,
    concurrency: usize,
    max_tasks: usize,
}

impl InteractiveContext {
    fn new() -> Self {
        Self {
            current_module: None,
            current_instance: None,
            module_name: None,
            options: HashMap::new(),
            concurrency: 4,
            max_tasks: 0,
        }
    }

    fn clear_module(&mut self) {
        self.current_module = None;
        self.current_instance = None;
        self.module_name = None;
        self.options.clear();
    }
}

/// Tab completion helper
struct CommandCompleter {
    commands: Vec<String>,
    show_subcommands: Vec<String>,
    modules: Vec<ModuleEntry>,
}

impl CommandCompleter {
    fn new() -> Self {
        Self {
            commands: vec![
                "use".into(),
                "show".into(),
                "set".into(),
                "unset".into(),
                "run".into(),
                "info".into(),
                "back".into(),
                "clear".into(),
                "help".into(),
                "exit".into(),
                "quit".into(),
            ],
            show_subcommands: vec!["modules".into(), "options".into()],
            modules: list_modules(),
        }
    }

    fn refresh_modules(&mut self) {
        self.modules = list_modules();
    }
}

impl Completer for CommandCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line_to_cursor = &line[..pos];
        let parts: Vec<&str> = line_to_cursor.split_whitespace().collect();

        match parts.as_slice() {
            // Complete command names at start
            [] => {
                let matches: Vec<Pair> = self
                    .commands
                    .iter()
                    .map(|c| Pair {
                        display: c.clone(),
                        replacement: c.clone(),
                    })
                    .collect();
                Ok((0, matches))
            }

            // Partial command at start
            [partial] if !line_to_cursor.ends_with(' ') => {
                let matches: Vec<Pair> = self
                    .commands
                    .iter()
                    .filter(|c| c.starts_with(partial))
                    .map(|c| Pair {
                        display: c.clone(),
                        replacement: c.clone(),
                    })
                    .collect();
                Ok((0, matches))
            }

            // Complete module names after "use "
            ["use"] if line_to_cursor.ends_with(' ') => {
                let matches: Vec<Pair> = self
                    .modules
                    .iter()
                    .map(|m| Pair {
                        display: m.name.clone(),
                        replacement: m.name.clone(),
                    })
                    .collect();
                Ok((4, matches)) // 4 = "use ".len()
            }

            ["use", partial] => {
                let start = line_to_cursor.find(' ').map(|i| i + 1).unwrap_or(0);
                let matches: Vec<Pair> = self
                    .modules
                    .iter()
                    .filter(|m| m.name.starts_with(partial))
                    .map(|m| Pair {
                        display: m.name.clone(),
                        replacement: m.name.clone(),
                    })
                    .collect();
                Ok((start, matches))
            }

            // Complete "modules" or "options" after "show "
            ["show"] if line_to_cursor.ends_with(' ') => {
                let matches: Vec<Pair> = self
                    .show_subcommands
                    .iter()
                    .map(|s| Pair {
                        display: s.clone(),
                        replacement: s.clone(),
                    })
                    .collect();
                Ok((5, matches)) // 5 = "show ".len()
            }

            ["show", partial] => {
                let start = line_to_cursor.find(' ').map(|i| i + 1).unwrap_or(0);
                let matches: Vec<Pair> = self
                    .show_subcommands
                    .iter()
                    .filter(|s| s.starts_with(partial))
                    .map(|s| Pair {
                        display: s.clone(),
                        replacement: s.clone(),
                    })
                    .collect();
                Ok((start, matches))
            }

            _ => Ok((pos, vec![])),
        }
    }
}

impl Hinter for CommandCompleter {
    type Hint = String;
}

impl Highlighter for CommandCompleter {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Borrowed(hint)
    }
}

impl Validator for CommandCompleter {}

impl Helper for CommandCompleter {}

/// Print the interactive console banner
fn print_interactive_banner() {
    println!(
        r#"
888     888     d8888 888      8888888b.         d8888 8888888b.        d8888 8888888b.
888     888    d88888 888      888   Y88b       d88888 888  'Y88b      d88888 888   Y88b
888     888   d88P888 888      888    888      d88P888 888    888     d88P888 888    888
Y88b   d88P  d88P 888 888      888   d88P     d88P 888 888    888    d88P 888 888   d88P
 Y88b d88P  d88P  888 888      8888888P'     d88P  888 888    888   d88P  888 8888888P'
  Y88o88P  d88P   888 888      888 T88b     d88P   888 888    888  d88P   888 888 T88b
   Y888P  d8888888888 888      888  T88b   d8888888888 888  .d88P d8888888888 888  T88b
    Y8P  d88P     888 88888888 888   T88b d88P     888 8888888P' d88P     888 888   T88b

    Interactive Console - Type 'help' for commands
"#
    );
}

/// Run the interactive console
pub fn run_interactive() -> Result<()> {
    print_interactive_banner();

    let mut ctx = InteractiveContext::new();
    let completer = CommandCompleter::new();

    let config = rustyline::Config::builder()
        .history_ignore_space(true)
        .completion_type(rustyline::CompletionType::List)
        .build();

    let mut rl: Editor<CommandCompleter, DefaultHistory> = Editor::with_config(config)?;
    rl.set_helper(Some(completer));

    // Load history
    let history_path = my_home()
        .ok()
        .flatten()
        .map(|p| p.join(".valradar_history"));
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    loop {
        let prompt = match &ctx.module_name {
            Some(name) => format!("valradar(\x1b[1;31m{}\x1b[0m) > ", name),
            None => "valradar > ".to_string(),
        };

        match rl.readline(&prompt) {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);

                match parse_command(&line) {
                    Command::Use(module) => cmd_use(&mut ctx, &module),
                    Command::ShowModules => {
                        // Refresh module list before showing
                        if let Some(helper) = rl.helper_mut() {
                            helper.refresh_modules();
                        }
                        cmd_show_modules();
                    }
                    Command::ShowOptions => cmd_show_options(&ctx),
                    Command::Info => cmd_info(&ctx),
                    Command::Set(opt, val) => cmd_set(&mut ctx, &opt, &val),
                    Command::Unset(opt) => cmd_unset(&mut ctx, &opt),
                    Command::Run(targets) => cmd_run(&mut ctx, targets),
                    Command::Back => cmd_back(&mut ctx),
                    Command::Help => cmd_help(),
                    Command::Clear => cmd_clear(),
                    Command::Exit => break,
                    Command::Empty => continue,
                    Command::Unknown(msg) => println!("[-] {}", msg),
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C - just continue
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D - exit
                println!();
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }

    // Save history
    if let Some(path) = history_path {
        let _ = rl.save_history(&path);
    }

    println!("Goodbye!");
    Ok(())
}

// Command handlers

fn cmd_use(ctx: &mut InteractiveContext, module_path: &str) {
    // Convert dot notation to file path
    let file_path = module_path.replace('.', "/") + ".py";

    match search_module(&file_path) {
        Some(path) => {
            let name = module_path.split('.').last().unwrap_or("main");
            let plugin = Plugin::new(name.to_string(), path);

            match plugin.instantiate() {
                Ok(instance) => {
                    debug("Plugin instantiated in interactive mode");
                    ctx.current_module = Some(plugin);
                    ctx.current_instance = Some(instance);
                    ctx.module_name = Some(module_path.to_string());
                    ctx.options.clear();
                    println!("[+] Loaded module: {}", module_path);
                }
                Err(e) => println!("[-] Failed to load module: {}", e),
            }
        }
        None => println!("[-] Module not found: {}", module_path),
    }
}

fn cmd_show_modules() {
    let modules = list_modules();

    if modules.is_empty() {
        println!("[-] No modules found");
        println!("    Modules are loaded from:");
        println!("    - ./modules/");
        println!("    - ~/.valradar/modules/");
        return;
    }

    println!();
    println!("Available modules:");
    println!("{}", "=".repeat(60));

    for entry in &modules {
        println!("  {}", entry.name);
    }

    println!();
    println!("{} module(s) found", modules.len());
    println!();
}

fn cmd_show_options(ctx: &InteractiveContext) {
    let plugin = match &ctx.current_module {
        Some(p) => p,
        None => {
            println!("[-] No module loaded. Use 'use <module>' first.");
            return;
        }
    };

    match plugin.get_info() {
        Ok(info) => {
            println!();
            println!("Module: {} v{}", info.name, info.version);
            if !info.description.is_empty() {
                println!("{}", info.description);
            }
            println!();

            if info.options.is_empty() {
                println!("No configurable options");
            } else {
                println!(
                    "{:<20} {:<25} {:<10} {}",
                    "Name", "Current", "Required", "Description"
                );
                println!("{}", "-".repeat(80));

                for opt in &info.options {
                    let current = ctx
                        .options
                        .get(&opt.name)
                        .map(|s| {
                            if s.len() > 22 {
                                format!("{}...", &s[..19])
                            } else {
                                s.clone()
                            }
                        })
                        .unwrap_or_default();

                    let req = if opt.required { "yes" } else { "no" };
                    println!("{:<20} {:<25} {:<10} {}", opt.name, current, req, opt.help);
                }
            }
            println!();
        }
        Err(e) => println!("[-] Error getting options: {}", e),
    }
}

fn cmd_info(ctx: &InteractiveContext) {
    let plugin = match &ctx.current_module {
        Some(p) => p,
        None => {
            println!("[-] No module loaded. Use 'use <module>' first.");
            return;
        }
    };

    match plugin.get_info() {
        Ok(info) => {
            println!();
            println!("       Name: {}", info.name);
            println!("    Version: {}", info.version);
            if !info.description.is_empty() {
                println!("Description: {}", info.description);
            }
            if !info.author.is_empty() {
                println!("     Author: {}", info.author);
            }
            println!();
        }
        Err(e) => println!("[-] Error getting info: {}", e),
    }
}

fn cmd_set(ctx: &mut InteractiveContext, opt: &str, value: &str) {
    ctx.options.insert(opt.to_string(), value.to_string());
    println!("{} => {}", opt, value);
}

fn cmd_unset(ctx: &mut InteractiveContext, opt: &str) {
    if ctx.options.remove(opt).is_some() {
        println!("Unset {}", opt);
    } else {
        println!("[-] Option '{}' was not set", opt);
    }
}

fn cmd_run(ctx: &mut InteractiveContext, targets: Vec<String>) {
    // Verify module is loaded
    if ctx.current_module.is_none() {
        println!("[-] No module loaded. Use 'use <module>' first.");
        return;
    }

    if ctx.current_instance.is_none() {
        println!("[-] Module not properly initialized");
        return;
    }

    if targets.is_empty() {
        println!("[-] No targets specified. Usage: run <target1> [target2...]");
        return;
    }

    println!("[*] Starting scan with {} target(s)...", targets.len());

    let bar = ProgressBar::new_spinner();
    bar.enable_steady_tick(std::time::Duration::from_millis(100));
    bar.set_message("Scanning...");

    // We need to clone plugin for the orchestrator
    // Create a new plugin instance since we can't clone the existing one
    let module_name = ctx.module_name.as_ref().unwrap();
    let file_path = module_name.replace('.', "/") + ".py";
    let path = match search_module(&file_path) {
        Some(p) => p,
        None => {
            bar.finish_and_clear();
            println!("[-] Module path not found");
            return;
        }
    };

    let name = module_name.split('.').last().unwrap_or("main");
    let new_plugin = Plugin::new(name.to_string(), path);
    let new_instance = match new_plugin.instantiate() {
        Ok(i) => i,
        Err(e) => {
            bar.finish_and_clear();
            println!("[-] Failed to instantiate plugin: {}", e);
            return;
        }
    };

    let mut orchestrator = Orchestrator::new(new_plugin, new_instance, ctx.concurrency);

    if ctx.max_tasks > 0 {
        orchestrator.set_max_tasks(ctx.max_tasks);
    }

    if !ctx.options.is_empty() {
        orchestrator.set_kwargs(ctx.options.clone());
    }

    let start_time = Instant::now();

    match orchestrator.run(targets) {
        Ok(results) => {
            let elapsed = start_time.elapsed();
            let tasks_processed = orchestrator.tasks_processed();
            let targets_visited = orchestrator.targets_visited();

            bar.set_style(ProgressStyle::with_template("{prefix} {msg}").unwrap());
            bar.set_prefix("[+]");
            bar.set_message(format!(
                "Completed: {} results, {} tasks processed, {} unique targets in {:.2}s",
                results.len(),
                tasks_processed,
                targets_visited,
                elapsed.as_secs_f64()
            ));
            bar.finish();

            // Display results
            if !results.is_empty() {
                println!();
                println!("{}", crate::utils::ProcessedData::new(results));
            }
        }
        Err(e) => {
            bar.finish_and_clear();
            println!("[-] Scan failed: {}", e);
        }
    }
}

fn cmd_back(ctx: &mut InteractiveContext) {
    if ctx.module_name.is_some() {
        let name = ctx.module_name.clone().unwrap_or_default();
        ctx.clear_module();
        println!("[*] Unloaded module: {}", name);
    } else {
        println!("[-] No module loaded");
    }
}

fn cmd_help() {
    println!(
        r#"
Commands:
  use <module>         Load a module (e.g., use modules.web.emails)
  show modules         List all available modules
  show options         Show options for current module
  info                 Show detailed info about current module
  set <opt> <value>    Set an option value
  unset <opt>          Clear an option value
  run <targets...>     Run the module against one or more targets
  back                 Unload current module
  clear                Clear the screen
  help                 Show this help message
  exit                 Exit the console

Examples:
  use modules.web.regex
  set pattern emails=[a-z]+@[a-z]+\.[a-z]+
  run https://example.com
"#
    );
}

fn cmd_clear() {
    let _ = execute!(std::io::stdout(), terminal::Clear(terminal::ClearType::All));
    let _ = execute!(
        std::io::stdout(),
        crossterm::cursor::MoveTo(0, 0)
    );
}
