"""DSCode binary wrapper — downloads and launches the native DSCode application."""

import sys
from .install import ensure_binary, get_binary_path


def main():
    """Entry point for the `dscode` CLI command."""
    binary = ensure_binary()
    import subprocess
    subprocess.run([binary] + sys.argv[1:])
