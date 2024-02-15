"""
Constructs that record important game events that occur during the demo.
"""

from numpy import uint8
from typing import List

from .game  import Team
from .      import UserID, EntityID, DemoTick

class Kill:
    """By God, he's dead!
    
    In all seriousness, this is a record of a kill."""

    @property
    def dead_id(self) -> UserID:
        """User ID of the killed / dead player."""
    @property
    def dead_entity(self) -> EntityID:
        """Entity ID of the killed / dead player."""
    @property
    def attacker_id(self) -> UserID:
        """User ID of the player was responsible for the final blow.
        May not be a valid ID if the player died to the environment."""
    @property
    def inflictor_id(self) -> EntityID:
        """Entity ID of the thing responsible for the final blow. May not be a player."""
    @property
    def weapon(self) -> str: ...
    @property
    def assister(self) -> UserID | None:
        """User ID of the assister. Could be none."""
    @property
    def dead_rocketjumping(self) -> bool:
        """If the killed/dead player was rocket jumping when they died."""
    @property
    def tick(self) -> DemoTick: ...

class Capture:
    @property
    def cp_index(self) -> uint8:
        """The index of the capture point. 0 is typically blue last or the
        midpoint in KOTH, with 4 being red last, but this is not guaranteed.
        The index of each point will be in order though."""
        ...

    @property
    def cp_name(self) -> str:
        """Name of the control point."""
        ...

    @property
    def team(self) -> Team:
        """The team that captured the point."""
        ...

    @property
    def cappers(self) -> List[UserID]:
        """Who was standing on the point when it was captured."""
        ...

    @property
    def tick(self) -> DemoTick:
        """When the capture occurred"""
        ...