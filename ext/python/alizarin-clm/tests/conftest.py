"""
Pytest configuration for CLM extension tests.
"""

import sys
from pathlib import Path

# Add the python package to the path for imports
python_pkg = Path(__file__).parent.parent / "python"
sys.path.insert(0, str(python_pkg))

# Also add the main alizarin package
alizarin_pkg = Path(__file__).parent.parent.parent.parent.parent / "crates" / "alizarin-python" / "python"
sys.path.insert(0, str(alizarin_pkg))
