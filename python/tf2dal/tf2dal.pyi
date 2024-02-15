# demo_analysis_lib
# Contains the interface for items in the library.

# Types contained in this file are considered basic; essentially, they don't depend
# on each other (or barely depend on each other).

"""
Demo Analysis Library
"""

print("Importing Demo Analysis Library!")

from typing import List, TypeAlias
from numpy import uint32, uint16

from .demo  import DemoData

DemoTick: TypeAlias = uint32
EntityID: TypeAlias = uint32
UserID: TypeAlias = uint16

## Demo Loading

def load_demo(file_path: str) -> DemoData:
    """Loads a demo and returns the associated data."""
    ...

def load_demo_rounds(file_path: str) -> List[DemoData]:
    """Loads a demo and returns data split into rounds.
    Each DemoData in the list should be a seperate round."""
    ...
