# CLI Reference

Complete reference for all Valradar command-line commands and options.

## Overview

```
valradar [OPTIONS] <COMMAND>

Commands:
  run      Run a plugin
  new      Create a new plugin from template
  list     List available plugins
  install  Install plugin dependencies
  doctor   Validate plugin structure

Options:
  -!, --debug    Enable debug mode
  -l, --license  Show license
  -h, --help     Print help
  -V, --version  Print version
```

## Commands

### run

Run a plugin with specified options.

```
valradar run [OPTIONS] <PLUGIN> [-- <ARGS>...]
```

**Aliases:** `r`

**Arguments:**
- `PLUGIN` — Plugin module name (e.g., `examples.emails`)
- `ARGS` — Arguments passed to the plugin's `init()` method

**Options:**

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--concurrency` | `-c` | Number of parallel workers | 4 |
| `--depth` | `-d` | Recursive processing depth | 1 |
| `--info` | `-i` | Show plugin info and exit | false |
| `--debug` | `-!` | Enable debug output | false |

**Examples:**

```bash
# Basic usage
valradar run examples.emails https://example.com

# With 8 workers
valradar run -c 8 examples.emails https://example.com

# Crawl 3 levels deep
valradar run -d 3 examples.emails https://example.com

# Combined options
valradar run -c 8 -d 3 examples.emails https://example.com

# Multiple arguments to plugin
valradar run examples.emails https://site1.com https://site2.com

# Show plugin info only
valradar run -i examples.emails

# Debug mode
valradar run -! examples.emails https://example.com
```

**Concurrency (`-c`):**

Controls how many worker processes run in parallel. Higher values can speed up processing but use more system resources.

- **Low (1-2):** Good for rate-limited APIs or debugging
- **Medium (4-8):** Default, balanced performance
- **High (8-16):** Fast processing, high resource usage

**Depth (`-d`):**

Controls how many recursive iterations to perform. Each iteration processes contexts returned by `collect()`.

- **1:** Only process initial contexts from `init()`
- **2:** Process initial + one level of collected contexts
- **3+:** Continue recursively up to the specified depth

---

### new

Create a new plugin from the built-in template.

```
valradar new [OPTIONS] <NAME>
```

**Aliases:** `n`

**Arguments:**
- `NAME` — Name for the new plugin

**Options:**

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--path` | `-p` | Output directory | Current directory |

**Examples:**

```bash
# Create in current directory
valradar new email-finder

# Create in specific directory
valradar new email-finder --path ./my-plugins

# Creates: ./email-finder.py (or ./my-plugins/email-finder.py)
```

**Generated Template:**

The template includes:
- Plugin class with metadata placeholders
- `init()`, `collect()`, and `process()` methods with examples
- TODO comments for customization
- Proper export (`VALRADAR_CONFIG = to_config(plugin)`)

---

### list

List all available plugins that Valradar can discover.

```
valradar list
```

**Aliases:** `ls`

**Output:**

```
┌──────────────────┬─────────────────────────────────┬─────────┐
│ Module           │ Name / Description              │ Version │
├──────────────────┼─────────────────────────────────┼─────────┤
│ examples.emails  │ emails                          │ 0.1.0   │
│                  │ Extract emails from a website   │         │
├──────────────────┼─────────────────────────────────┼─────────┤
│ examples.yara-..│ yara-scan                       │ 0.1.0   │
│                  │ Scan content with YARA rules    │         │
└──────────────────┴─────────────────────────────────┴─────────┘
```

**Discovery Paths:**

Valradar searches for plugins in these locations:
1. `./examples/` — Example plugins
2. `./modules/` — Local plugins
3. `~/.valradar/modules/` — User plugins

---

### install

Install Python dependencies for a plugin.

```
valradar install <PLUGIN>
```

**Aliases:** `i`

**Arguments:**
- `PLUGIN` — Plugin module name

**Example:**

```bash
valradar install examples.emails
```

**How it works:**

1. Loads the plugin's `VALRADAR_CONFIG`
2. Looks for a `dependencies` list in metadata
3. Runs `pip install` for each dependency

**Adding dependencies to your plugin:**

```python
class MyPlugin(Plugin):
    name = "my-plugin"
    # ...
    
    # Add this to enable install command
    dependencies = ["requests", "beautifulsoup4>=4.9"]
```

---

### doctor

Validate a plugin's structure and check for common issues.

```
valradar doctor <PLUGIN>
```

**Arguments:**
- `PLUGIN` — Plugin module name

**Example:**

```bash
valradar doctor examples.emails
```

**Output:**

```
Checking plugin: examples.emails
Path: ./examples/emails.py

┌─────────────────────────┬────────────────────────────────────────┐
│ Check                   │ Result                                 │
├─────────────────────────┼────────────────────────────────────────┤
│ ✓ File readable         │ Plugin file can be read                │
│ ✓ Python syntax         │ Module loads without errors            │
│ ✓ VALRADAR_CONFIG       │ Config dictionary found                │
│ ✓ init function         │ Present and callable                   │
│ ✓ collect_data function │ Present and callable                   │
│ ✓ process_data function │ Present and callable                   │
│ ✓ Metadata              │ name: emails, version: 0.1.0           │
└─────────────────────────┴────────────────────────────────────────┘

Plugin is healthy! ✓
```

**Checks performed:**

1. **File readable** — Plugin file exists and can be read
2. **Python syntax** — No syntax errors, module loads
3. **VALRADAR_CONFIG** — Export dictionary exists
4. **init function** — `init` is present and callable
5. **collect_data function** — `collect_data` is present and callable
6. **process_data function** — `process_data` is present and callable
7. **Metadata** — Name, version, and description are present

---

## Global Options

These options work with any command:

### --debug (-!)

Enable debug mode for verbose output.

```bash
valradar --debug run examples.emails https://example.com
valradar -! run examples.emails https://example.com
```

### --license (-l)

Display license information.

```bash
valradar --license
```

### --help (-h)

Show help for Valradar or a specific command.

```bash
valradar --help
valradar run --help
valradar new --help
```

### --version (-V)

Show version information.

```bash
valradar --version
```

---

## Legacy Mode

For backward compatibility, you can run plugins without the `run` subcommand:

```bash
# Legacy (still works)
valradar examples.emails https://example.com

# Equivalent to
valradar run examples.emails https://example.com
```

However, legacy mode doesn't support all options. Use the `run` subcommand for full control.

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Plugin error (import failed, runtime error) |
| 2 | Invalid arguments |

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `VALRADAR_MODULE_PATH` | Additional directories to search for plugins (colon-separated) |
| `PYTHONPATH` | Affects Python module resolution |

---

## Tips

### Combining with Unix Tools

```bash
# Save output to file
valradar run examples.emails https://example.com > results.txt

# Filter results with grep
valradar run examples.emails https://example.com | grep "@gmail.com"

# Count results
valradar run examples.emails https://example.com | wc -l
```

### Performance Tuning

```bash
# For rate-limited targets, use low concurrency
valradar run -c 1 my-api-plugin api-key

# For high-throughput, increase workers
valradar run -c 16 examples.emails https://large-site.com

# Balance depth and concurrency
valradar run -c 4 -d 2 examples.emails https://example.com
```
