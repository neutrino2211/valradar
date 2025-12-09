import re

import bs4
import requests

# https://uibakery.io/regex-library/email-regex-python
pattern = "(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|\"(?:[\\x01-\\x08\\x0b\\x0c\\x0e-\\x1f\\x21\\x23-\\x5b\\x5d-\\x7f]|\\\\[\\x01-\\x09\\x0b\\x0c\\x0e-\\x7f])*\")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\\[(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?|[a-z0-9-]*[a-z0-9]:(?:[\\x01-\\x08\\x0b\\x0c\\x0e-\\x1f\\x21-\\x5a\\x53-\\x7f]|\\\\[\\x01-\\x09\\x0b\\x0c\\x0e-\\x7f])+)\\])"
headers = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
}


class DataContext:
    def __init__(self, url):
        self.url = url if url.endswith("/") else url + "/"
        self.data = {}
        self.emails = []

    def collect(self):
        # Do some work and store state
        if not self.url.startswith("http"):
            return []

        self.data["content"] = requests.get(self.url, headers=headers).text
        self.emails = re.findall(pattern, self.data["content"])

        return [DataContext(link) for link in self.extract_links()]

    def extract_links(self):
        soup = bs4.BeautifulSoup(self.data["content"], "html.parser")
        hrefs = []
        for link in soup.find_all("a"):
            href: str = link.get("href").__str__()
            if href:
                if href.startswith("/") or href.startswith("#"):
                    hrefs.append(self.url + href)
                else:
                    hrefs.append(href)
        return hrefs

    def process(self):
        if len(self.emails) > 0:
            return {"url": self.url[:80], "emails": ", ".join(self.emails)}
        else:
            return None


def _VALRADAR_INIT(args):
    if len(args) == 0:
        print("no urls provided")
        exit(1)

    return [DataContext(url) for url in args]


def _VALRADAR_COLLECT_DATA(context):
    return context.collect()


def _VALRADAR_PROCESS_DATA(context):
    return context.process()


VALRADAR_CONFIG = {
    "init": _VALRADAR_INIT,
    "collect_data": _VALRADAR_COLLECT_DATA,
    "process_data": _VALRADAR_PROCESS_DATA,
    "metadata": {
        "name": "Emails",
        "description": "Extract emails from a website",
        "version": "0.1.0",
        "tags": ["email", "scraping", "web"],
        "author": "John Doe",
        "license": "MIT",
        "url": "https://github.com/john-doe/emails",
        "dependencies": ["requests", "bs4"],
        "requirements": ["requests", "bs4"],
        "examples": ["https://example.com", "https://example.com/about"],
        "notes": "This plugin is a work in progress and may not work as expected.",
    },
}
