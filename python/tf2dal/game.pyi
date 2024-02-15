"""
Game-related classes that translate to generic information about the game.
Typically not directly related to players.
"""

from typing import List, Tuple
from enum import Enum
from numpy import uint8

from .      import DemoTick
from .math  import Vector

class World:
    """Defines the boundaries of the world as given by the demofile.
    You can expect bound_min.x < bound_max.x, and so on."""
    @property
    def bound_min(self) -> Vector: ...

    @property
    def bound_max(self) -> Vector: ...

class Class(Enum):
    Other = uint8(0)
    Scout = uint8(1)
    Sniper = uint8(2)
    Soldier = uint8(3)
    Demoman = uint8(4)
    Medic = uint8(5)
    Heavy = uint8(6)
    Pyro = uint8(7)
    Spy = uint8(8)
    Engineer = uint8(9)

class ClassList(List): 
    """Counts how many times a player has switched to a class.
    List items are tuples in the form of (class, count)."""
    def __len__(self) -> int:
        """Number of classes played by the player."""

    def __contains__(self, item: Class) -> bool:
        """Check to see if a player has played this class."""

    def __iter__(self): #TODO: iter?
        """Iterates over all classes that have been played."""

class ClassListIter:
    def __iter__(self) -> ClassListIter: ...
    def __next__(self) -> Tuple[Class, uint8] | None: ...

class Team(Enum):
    Other = uint8(0)
    Spectator = uint8(1)
    Red = uint8(2)
    Blue = uint8(3)

    def is_player(self) -> bool: 
        """If the team actually supports having alive players on it.
        The only teams to do so are Red and Blue."""
        ...

class Round:
    """Defines information on a round, such as the tick it started, ended,
    and who won."""
    @property
    def start_tick(self) -> DemoTick: ...
    @property
    def end_tick(self) -> DemoTick: ...
    @property
    def winner(self) -> Team: ...

    def is_tie(self) -> bool: ...