# Examples

This page walks through real plugin examples to demonstrate Valradar patterns and best practices.

## Email Extractor

The `examples.emails` plugin crawls websites and extracts email addresses. It's a great example of recursive data collection.

### Full Source

```python
"""Email extractor plugin using the Valradar SDK.

This plugin crawls websites and extracts email addresses from the HTML content.
It demonstrates the class-based Plugin API with Context for data management.

Usage:
    valradar examples.emails https://example.com
"""

import re
from typing import Any

import bs4

from valradar import Context, Plugin, to_config

# Email regex pattern
EMAIL_PATTERN = re.compile(
    r"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"
    r"\"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|"
    r"\\[\x01-\x09\x0b\x0c\x0e-\x7f])*\")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+"
    r"[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}"
    r"(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:"
    r"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"
)


class EmailsPlugin(Plugin):
    """Extract email addresses from websites."""

    name = "emails"
    description = "Extract emails from a website"
    version = "0.1.0"
    author = "Mainasara Tsowa"
    tags = ["email", "scraping", "web"]

    def init(self, args: list[str]) -> list[Context]:
        """Initialize with URLs from command line arguments."""
        if not args:
            print("Usage: valradar examples.emails <url> [url...]")
            exit(1)

        return [Context(url=self._normalize_url(url)) for url in args]

    def collect(self, ctx: Context) -> list[Context]:
        """Fetch URL content and extract links for recursive crawling."""
        url = ctx.get("url", "")

        # Skip non-HTTP URLs
        if not url.startswith("http"):
            return []

        # Fetch the page content
        try:
            response = ctx.fetch(url)
            content = response.text
        except Exception as e:
            print(f"Error fetching {url}: {e}")
            return []

        # Store content and extract emails
        ctx.set("content", content)
        ctx.set("emails", EMAIL_PATTERN.findall(content))

        # Extract and return links as new contexts
        links = self._extract_links(content, url)
        return [Context(url=link) for link in links]

    def process(self, ctx: Context) -> dict[str, Any] | None:
        """Return extracted emails if any were found."""
        emails = ctx.get("emails", [])

        if emails:
            return {
                "url": ctx.get("url", "")[:80],  # Truncate long URLs
                "emails": ", ".join(emails),
            }

        return None

    def _normalize_url(self, url: str) -> str:
        """Ensure URL ends with a trailing slash."""
        return url if url.endswith("/") else url + "/"

    def _extract_links(self, content: str, base_url: str) -> list[str]:
        """Extract all links from HTML content."""
        soup = bs4.BeautifulSoup(content, "html.parser")
        links: list[str] = []

        for anchor in soup.find_all("a"):
            href = anchor.get("href")
            if href:
                href_str = str(href)
                if href_str.startswith("/") or href_str.startswith("#"):
                    links.append(base_url + href_str)
                else:
                    links.append(href_str)

        return links


# Export for Valradar runtime
plugin = EmailsPlugin()
VALRADAR_CONFIG = to_config(plugin)
```

### Key Patterns

1. **Argument validation in init()** — Check for required arguments and show usage
2. **Error handling in collect()** — Catch exceptions to avoid crashing
3. **Storing data with set()** — Save fetched content and extracted data
4. **Returning new contexts** — Enable recursive crawling by returning link contexts
5. **Filtering in process()** — Only return results when emails are found
6. **Helper methods** — Keep main methods clean with private helpers

### Running the Plugin

```bash
# Basic usage
valradar run examples.emails https://example.com

# Deeper crawl
valradar run -d 3 examples.emails https://example.com

# Multiple starting URLs
valradar run examples.emails https://site1.com https://site2.com
```

---

## YARA Scanner

The `examples.yara-scan` plugin scans content with YARA rules. It demonstrates integration with external tools.

### Source Highlights

```python
class YaraScanPlugin(Plugin):
    name = "yara-scan"
    description = "Scan URLs with YARA rules"
    version = "0.1.0"
    tags = ["security", "yara", "scanning"]

    def init(self, args: list[str]) -> list[Context]:
        # First arg is YARA rule file, rest are URLs
        if len(args) < 2:
            print("Usage: valradar yara-scan <rules.yar> <url> [url...]")
            exit(1)
        
        rules_path = args[0]
        urls = args[1:]
        
        # Compile rules once, share across contexts
        rules = yara.compile(filepath=rules_path)
        
        return [Context(url=url, rules=rules) for url in urls]

    def collect(self, ctx: Context) -> list[Context]:
        response = ctx.fetch(ctx.get("url"))
        ctx.set("content", response.content)
        return []

    def process(self, ctx: Context) -> dict | None:
        rules = ctx.get("rules")
        content = ctx.get("content")
        
        matches = rules.match(data=content)
        
        if matches:
            return {
                "url": ctx.get("url"),
                "matches": [str(m) for m in matches]
            }
        return None
```

### Key Patterns

1. **Multiple argument types** — First arg is config, rest are targets
2. **Shared state** — Compile YARA rules once and share via context
3. **Binary content** — Use `response.content` for binary scanning

---

## Template: Minimal Plugin

Here's a template for starting new plugins:

```python
"""Description of what this plugin does.

Usage:
    valradar run my_plugin <arg> [arg...]
"""

from typing import Any
from valradar import Plugin, Context, to_config


class MyPlugin(Plugin):
    name = "my-plugin"
    description = "Brief description"
    version = "0.1.0"
    author = "Your Name"
    tags = ["category"]

    def init(self, args: list[str]) -> list[Context]:
        if not args:
            print("Usage: valradar run my_plugin <arg>")
            exit(1)
        return [Context(input=arg) for arg in args]

    def collect(self, ctx: Context) -> list[Context]:
        # Fetch/process data here
        # ctx.set("key", value)
        return []

    def process(self, ctx: Context) -> dict[str, Any] | None:
        return {
            "input": ctx.get("input"),
            # Add more fields
        }


plugin = MyPlugin()
VALRADAR_CONFIG = to_config(plugin)
```

---

## Template: Web Scraper

Template for web scraping plugins:

```python
"""Scrape data from web pages."""

from typing import Any
import bs4
from valradar import Plugin, Context, to_config


class ScraperPlugin(Plugin):
    name = "scraper"
    description = "Scrape data from web pages"
    version = "0.1.0"
    tags = ["web", "scraping"]

    def init(self, args: list[str]) -> list[Context]:
        return [Context(url=url) for url in args]

    def collect(self, ctx: Context) -> list[Context]:
        url = ctx.get("url")
        
        try:
            response = ctx.fetch(url)
            ctx.set("html", response.text)
            ctx.set("status", response.status_code)
        except Exception as e:
            ctx.set("error", str(e))
        
        return []

    def process(self, ctx: Context) -> dict[str, Any] | None:
        if ctx.has("error"):
            return {"url": ctx.get("url"), "error": ctx.get("error")}
        
        html = ctx.get("html", "")
        soup = bs4.BeautifulSoup(html, "html.parser")
        
        # Extract data with BeautifulSoup
        title = soup.title.string if soup.title else "N/A"
        
        return {
            "url": ctx.get("url"),
            "title": title,
            "status": ctx.get("status")
        }


plugin = ScraperPlugin()
VALRADAR_CONFIG = to_config(plugin)
```

---

## Template: API Client

Template for API-based plugins:

```python
"""Fetch data from an API."""

from typing import Any
from valradar import Plugin, Context, to_config


class APIPlugin(Plugin):
    name = "api-client"
    description = "Fetch data from REST API"
    version = "0.1.0"
    tags = ["api"]
    
    BASE_URL = "https://api.example.com"

    def init(self, args: list[str]) -> list[Context]:
        # args could be IDs, search terms, etc.
        return [Context(query=arg) for arg in args]

    def collect(self, ctx: Context) -> list[Context]:
        query = ctx.get("query")
        url = f"{self.BASE_URL}/search?q={query}"
        
        try:
            response = ctx.fetch(url)
            data = response.json()
            ctx.set("results", data.get("items", []))
        except Exception as e:
            ctx.set("error", str(e))
        
        return []

    def process(self, ctx: Context) -> dict[str, Any] | None:
        if ctx.has("error"):
            return None
        
        results = ctx.get("results", [])
        return {
            "query": ctx.get("query"),
            "count": len(results),
            "results": str(results[:5])  # First 5
        }


plugin = APIPlugin()
VALRADAR_CONFIG = to_config(plugin)
```

## Next Steps

- [CLI Reference](./cli-reference.md) — All available commands
- [API Reference](./api-reference.md) — Complete SDK documentation
