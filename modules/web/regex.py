import requests
import bs4
import re
import argparse

headers = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
}

class DataContext:
    def __init__(self, url, types, processed_urls = []):
        self.url = url if url.endswith("/") else url + "/"
        self.data = {}
        self.processed_urls = processed_urls
        self.types = types
        self.types_result = {}

    def collect(self):
        # Do some work and store state
        if not self.url.startswith('http'):
            return []
        
        if self.url in self.processed_urls:
            return []

        self.data['content'] = requests.get(self.url, headers=headers).text
        for k in self.types.keys():
            self.types_result[k] = re.findall(self.types[k], self.data['content'])
        
        return [DataContext(link, self.types, self.processed_urls) for link in self.extract_links()]
    
    def extract_links(self):
        if self.url in self.processed_urls:
            return []
        
        self.processed_urls.append(self.url)
        
        soup = bs4.BeautifulSoup(self.data['content'], 'html.parser')
        hrefs = []
        for link in soup.find_all('a'):
            href = link.get('href')
            if href:
                if href.startswith('/') or href.startswith("#"):
                    hrefs.append(self.url + href)
                else:
                    hrefs.append(href)
        return hrefs

    def process(self):
        if len(self.types_result.keys()) > 0:
            d = {"url": self.url[:80] }
            for k in self.types_result.keys():
                d[k] = ', '.join(self.types_result[k])
            return d
        else:
            return None

def _VALRADAR_INIT(args):
    parser = argparse.ArgumentParser("web.regex", description="D")
    parser.add_argument("url", help="The url to initiate scraping on")
    parser.add_argument("-t", "--type", help="A mapping of a type to a regex that matches it -t letters='[a-zA-Z]'", action="append", default=[])
    args = parser.parse_args(args)
    types_dict = { a.split("=")[0]:a.split("=")[1] for a in args.type }
    return [DataContext(args.url, types_dict)]

def _VALRADAR_COLLECT_DATA(context):
    return context.collect()

def _VALRADAR_PROCESS_DATA(context):
    return context.process()

VALRADAR_CONFIG = {
    "init": _VALRADAR_INIT,
    "collect_data": _VALRADAR_COLLECT_DATA,
    "process_data": _VALRADAR_PROCESS_DATA,
    "metadata": {
        "name": "Email Scraper",
        "description": "Extract emails from a website",
        "version": "0.0.1",
        "tags": ["email", "scraping", "web"],
        "author": "Mainasara Tsowa <tsowamainasara@gmail.com>",
        "license": "MIT",
        "url": "https://github.com/neutrino2211/valradar/blob/dev/modules/web/emails.py",
        "dependencies": ["requests", "bs4"]
    }
}