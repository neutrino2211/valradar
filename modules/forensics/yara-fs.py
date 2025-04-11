import os
import yara
import argparse

class YaraContext:
    def __init__(self, directory, rules):
        self.directory = directory
        self.rules = rules
        self.files = []

    def collect(self):
        directories = []
        
        try:
            for file in os.listdir(self.directory):
                if os.path.isfile(os.path.join(self.directory, file)):
                    self.files.append(os.path.join(self.directory, file))
                elif os.path.isdir(os.path.join(self.directory, file)):
                    directories.append(os.path.join(self.directory, file))
        except Exception as e:
            return []

        return [YaraContext(directory, self.rules) for directory in directories]

    def process(self):
        rule_matches = []
        for file in self.files:
            content = open(file, "rb").read()
            matches = self.rules.match(data=content)
            if matches:
                rule_matches.append(f'{os.path.basename(file)}: {", ".join([match.rule for match in matches])}')

        if len(rule_matches) > 0:
            return {"directory": self.directory, "matches": ', '.join(rule_matches)}
        else:
            return None


def _YARA_INIT(args):
    parser = argparse.ArgumentParser()
    parser.add_argument("--directory", type=str, required=True)
    parser.add_argument("--rules_file", type=str, required=True)
    args = parser.parse_args(args)
    rules = yara.compile(args.rules_file)
    return [YaraContext(args.directory, rules)]

def _YARA_COLLECT_DATA(context):
    return context.collect()

def _YARA_PROCESS_DATA(context):
    return context.process()

VALRADAR_CONFIG = {
    "init": _YARA_INIT,
    "collect_data": _YARA_COLLECT_DATA,
    "process_data": _YARA_PROCESS_DATA,
    "metadata": {
        "name": "YARA Filesystem Scanner",
        "description": "Deeply scan a filesystem with YARA rules",
        "version": "0.0.1",
        "tags": ["malware", "analysis", "yara", "forensics"],
        "author": "Mainasara Tsowa <tsowamainasara@gmail.com>",
        "license": "MIT",
        "url": "https://github.com/neutrino2211/valradar/blob/dev/modules/forensics/yara-fs.py",
        "dependencies": ["yara-python"]    
    }
}