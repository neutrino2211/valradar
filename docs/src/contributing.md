# Contributing

Thank you for your interest in contributing to Valradar! This guide will help you get started.

## Ways to Contribute

- 🐛 **Report bugs** — Found an issue? Open a GitHub issue
- 💡 **Suggest features** — Have an idea? Start a discussion
- 📖 **Improve docs** — Fix typos, add examples, clarify explanations
- 🔌 **Write plugins** — Share useful plugins with the community
- 🛠️ **Submit code** — Fix bugs or implement features

## Getting Started

### 1. Fork and Clone

```bash
# Fork on GitHub, then clone your fork
git clone https://github.com/YOUR-USERNAME/valradar.git
cd valradar
```

### 2. Set Up Development Environment

**Rust:**

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build
```

**Python SDK:**

```bash
cd python
pip install -e ".[dev]"
```

### 3. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-description
```

## Code Style

### Rust

Follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy
```

### Python

Follow PEP 8 with these preferences:

- Line length: 100 characters
- Use type hints
- Use double quotes for strings

```bash
# Format with black
black python/

# Sort imports
isort python/

# Type check
mypy python/valradar
```

## Commit Messages

Use conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat` — New feature
- `fix` — Bug fix
- `docs` — Documentation changes
- `style` — Formatting, no code change
- `refactor` — Code restructuring
- `test` — Adding/updating tests
- `chore` — Maintenance tasks

**Examples:**

```
feat(cli): add --json output option for run command

fix(plugin): handle empty response in collect phase

docs(readme): add installation instructions for Windows

refactor(orchestrator): simplify worker pool management
```

## Pull Request Process

### 1. Before Submitting

- [ ] Code compiles without warnings (`cargo build`)
- [ ] Tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`, `black python/`)
- [ ] Documentation is updated if needed
- [ ] Commit messages follow conventions

### 2. Submit PR

1. Push your branch to your fork
2. Open a Pull Request against `main`
3. Fill out the PR template
4. Link any related issues

### 3. Review Process

- Maintainers will review your PR
- Address any requested changes
- Once approved, your PR will be merged

## Development Guidelines

### Adding CLI Commands

1. Create a new file in `src/commands/`
2. Add the command to `src/commands/mod.rs`
3. Register in `src/main.rs` under the `Commands` enum
4. Update CLI documentation

### Modifying the Python SDK

1. Make changes in `python/valradar/`
2. Update type hints
3. Add/update docstrings
4. Update API reference docs if needed

### Writing Tests

**Rust tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature() {
        // Arrange
        // Act
        // Assert
    }
}
```

**Python tests:**

```python
def test_context_get_set():
    ctx = Context(url="https://example.com")
    ctx.set("key", "value")
    assert ctx.get("key") == "value"
```

## Documentation

### Building Docs Locally

```bash
cd docs
mdbook serve --open
```

### Documentation Structure

- `introduction.md` — Overview and key features
- `getting-started.md` — Setup process overview
- `installation.md` — Detailed installation steps
- `quick-start.md` — First plugin in 5 minutes
- `writing-plugins.md` — Plugin development guide
- `plugin-structure.md` — Plugin class reference
- `context.md` — Context class reference
- `examples.md` — Plugin examples
- `cli-reference.md` — CLI commands
- `api-reference.md` — Python SDK API
- `contributing.md` — This file

### Writing Good Documentation

- Use clear, concise language
- Include code examples
- Add cross-references to related pages
- Test all code examples work

## Project Structure

```
valradar/
├── src/                  # Rust source code
│   ├── main.rs           # CLI entry point
│   ├── commands/         # CLI commands
│   ├── orchestrator.rs   # Worker coordination
│   ├── plugin.rs         # Plugin loader
│   └── utils/            # Helper functions
├── python/               # Python SDK
│   └── valradar/
│       ├── __init__.py   # SDK exports
│       ├── plugin.py     # Plugin base class
│       └── context.py    # Context class
├── examples/             # Example plugins
├── modules/              # User plugins (gitignored)
├── docs/                 # mdBook documentation
└── .github/              # GitHub Actions workflows
```

## Issue Guidelines

### Bug Reports

Include:
- Valradar version (`valradar --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Error messages (full output)

### Feature Requests

Include:
- Use case description
- Proposed solution
- Alternative solutions considered

## Community

- **GitHub Issues** — Bug reports and feature requests
- **GitHub Discussions** — Questions and ideas
- **Pull Requests** — Code contributions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Valradar! 🎉
