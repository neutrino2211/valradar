"""YARA filesystem scanner - recursively scans directories with YARA rules."""

import os

from valradar.sdk import Module, Option, Result, Task

# YARA import is optional - fail gracefully if not installed
try:
    import yara
    YARA_AVAILABLE = True
except ImportError:
    YARA_AVAILABLE = False


class YaraScanner(Module):
    """Recursively scans a filesystem directory with YARA rules."""

    name = "YARA Filesystem Scanner"
    description = "Deeply scan a filesystem with YARA rules"
    author = "Mainasara Tsowa <tsowamainasara@gmail.com>"
    version = "0.1.0"
    options = [
        Option("directory", type="str", required=True, help="Directory to scan"),
        Option("rules_file", type="str", required=True, help="Path to YARA rules file"),
    ]

    def setup(self):
        """Initialize the YARA rules compiler."""
        if not YARA_AVAILABLE:
            raise ImportError("yara-python is required for this module. Install with: pip install yara-python")
        self.rules = None
        self.rules_file = None

    def run(self, target: str, **kwargs):
        """
        Scan a directory for YARA rule matches.

        Args:
            target: Directory path to scan
            **kwargs: May contain 'rules_file' path

        Yields:
            Result: When YARA rules match files in the directory
            Task: For each subdirectory found
        """
        # Load rules if not already loaded
        rules_file = kwargs.get('rules_file', self.rules_file)
        if rules_file and self.rules is None:
            try:
                self.rules = yara.compile(rules_file)
                self.rules_file = rules_file
            except Exception as e:
                return

        if self.rules is None:
            return

        # Ensure target is a valid directory
        if not os.path.isdir(target):
            return

        try:
            entries = os.listdir(target)
        except (PermissionError, OSError):
            return

        # Collect files and subdirectories
        files = []
        subdirs = []

        for entry in entries:
            full_path = os.path.join(target, entry)
            if os.path.isfile(full_path):
                files.append(full_path)
            elif os.path.isdir(full_path):
                subdirs.append(full_path)

        # Scan files in this directory
        rule_matches = []
        for file_path in files:
            try:
                with open(file_path, "rb") as f:
                    content = f.read()
                matches = self.rules.match(data=content)
                if matches:
                    match_names = ", ".join(m.rule for m in matches)
                    rule_matches.append(f"{os.path.basename(file_path)}: {match_names}")
            except (PermissionError, OSError, IOError):
                continue

        # Yield results if matches found
        if rule_matches:
            yield Result(
                host=target,
                data={"matches": ", ".join(rule_matches)}
            )

        # Yield tasks for subdirectories
        for subdir in subdirs:
            yield Task(target=subdir, kwargs={'rules_file': self.rules_file})


# Export for Rust loader
MODULE_CLASS = YaraScanner
