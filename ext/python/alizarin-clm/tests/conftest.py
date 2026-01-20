"""
Pytest configuration for CLM extension tests.
"""

import sys
from pathlib import Path

import pytest

# Add the python package to the path for imports
python_pkg = Path(__file__).parent.parent / "python"
sys.path.insert(0, str(python_pkg))

# Also add the main alizarin package
alizarin_pkg = Path(__file__).parent.parent.parent.parent.parent / "crates" / "alizarin-python" / "python"
sys.path.insert(0, str(alizarin_pkg))


@pytest.fixture(autouse=True)
def setup_rdm_namespace():
    """
    Set up the RDM namespace before each test and clean up after.

    This is required for tests that use from_labels() without explicit IDs.
    """
    import alizarin

    # Set up namespace for deterministic UUID generation
    alizarin.set_rdm_namespace("http://test.example.org/rdm/")

    yield

    # Clean up after test
    try:
        alizarin.clear_global_rdm_cache()
    except Exception:
        pass
    try:
        alizarin.clear_graphs()
    except Exception:
        pass
