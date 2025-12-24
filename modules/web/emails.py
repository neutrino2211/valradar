"""Email scraper module - extracts email addresses from web pages."""

import re
from urllib.parse import urljoin, urlparse

import requests
from bs4 import BeautifulSoup

from valradar.sdk import Module, Option, Result, Task

# Email regex pattern
EMAIL_PATTERN = r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"


class EmailScanner(Module):
    """Scans web pages for email addresses and crawls links."""

    name = "Email Scraper"
    description = "Extract email addresses from web pages"
    author = "Valradar"
    version = "0.1.0"
    options = []

    def setup(self):
        """Initialize the HTTP session."""
        self.session = requests.Session()
        self.session.headers.update(
            {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                "AppleWebKit/537.36 (KHTML, like Gecko) "
                "Chrome/120.0.0.0 Safari/537.36"
            }
        )

    def run(self, target: str, **kwargs):
        """
        Scan a URL for emails and yield new links to crawl.

        Args:
            target: URL to scan

        Yields:
            Result: When emails are found
            Task: For each link found on the page
        """
        # Normalize URL
        if not target.startswith(("http://", "https://")):
            target = "https://" + target

        try:
            response = self.session.get(target, timeout=10)
            response.raise_for_status()
        except requests.RequestException as e:
            # Log error but don't crash - just skip this target
            return

        # Extract emails
        emails = list(set(re.findall(EMAIL_PATTERN, response.text)))
        if emails:
            yield Result(
                host=target[:80],  # Truncate for display
                data={"emails": ", ".join(emails)},
            )

        # Extract and yield links for crawling
        base_domain = urlparse(target).netloc
        soup = BeautifulSoup(response.text, "html.parser")

        for link in soup.find_all("a", href=True):
            href = link["href"]

            # Skip empty, anchor-only, and javascript links
            if not href or href.startswith(("#", "javascript:", "mailto:", "tel:")):
                continue

            # Resolve relative URLs
            full_url = urljoin(target, href)

            # Only follow links on the same domain
            link_domain = urlparse(full_url).netloc
            if link_domain == base_domain:
                yield Task(target=full_url)


# Export for Rust loader
MODULE_CLASS = EmailScanner
