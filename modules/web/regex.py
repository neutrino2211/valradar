"""Regex pattern scanner - searches web pages for custom regex patterns."""

import re
from urllib.parse import urljoin, urlparse

import requests
from bs4 import BeautifulSoup

from valradar.sdk import Module, Option, Result, Task


class RegexScanner(Module):
    """Scans web pages for custom regex patterns and crawls links."""

    name = "Regex Scanner"
    description = "Search web pages for custom regex patterns"
    author = "Mainasara Tsowa <tsowamainasara@gmail.com>"
    version = "0.1.0"
    options = [
        Option(
            "pattern",
            type="str",
            required=False,
            help="Regex pattern to search for (name=pattern format)",
        ),
    ]

    def setup(self):
        """Initialize the HTTP session and pattern storage."""
        self.session = requests.Session()
        self.session.headers.update(
            {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                "AppleWebKit/537.36 (KHTML, like Gecko) "
                "Chrome/120.0.0.0 Safari/537.36"
            }
        )
        # Default patterns - can be extended via CLI
        self.patterns = {}

    def run(self, target: str, **kwargs):
        """
        Scan a URL for regex patterns and yield new links to crawl.

        Args:
            target: URL to scan
            **kwargs: May contain 'pattern' in format 'name=regex'

        Yields:
            Result: When patterns are matched
            Task: For each link found on the page
        """
        # Parse patterns from kwargs if provided
        if "pattern" in kwargs and kwargs["pattern"]:
            pattern_str = kwargs["pattern"]
            if "=" in pattern_str:
                name, regex = pattern_str.split("=", 1)
                self.patterns[name] = regex

        # Normalize URL
        if not target.startswith(("http://", "https://")):
            target = "https://" + target

        try:
            response = self.session.get(target, timeout=10)
            response.raise_for_status()
        except requests.RequestException:
            return

        # Search for patterns
        matches = {}
        for name, pattern in self.patterns.items():
            found = list(set(re.findall(pattern, response.text)))
            if found:
                matches[name] = ", ".join(found[:10])  # Limit to first 10 matches

        if matches:
            yield Result(host=target[:80], data=matches)

        # Extract and yield links for crawling
        base_domain = urlparse(target).netloc
        soup = BeautifulSoup(response.text, "html.parser")

        for link in soup.find_all("a", href=True):
            href = link["href"]

            if not href or href.startswith(("#", "javascript:", "mailto:", "tel:")):
                continue

            full_url = urljoin(target, href)
            link_domain = urlparse(full_url).netloc

            if link_domain == base_domain:
                # Pass the pattern along to child tasks
                yield Task(target=full_url, kwargs=kwargs)


# Export for Rust loader
MODULE_CLASS = RegexScanner
