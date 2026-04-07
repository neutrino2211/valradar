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

# Email regex pattern from https://uibakery.io/regex-library/email-regex-python
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
                    # Relative URL - prepend base
                    links.append(base_url + href_str)
                else:
                    links.append(href_str)

        return links


# Instantiate and export for Valradar runtime
plugin = EmailsPlugin()
VALRADAR_CONFIG = to_config(plugin)
