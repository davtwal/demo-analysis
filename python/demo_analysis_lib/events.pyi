from numpy import uint32, uint16#, float32, uint8
#from enum import Enum
#from typing import List, Dict

from demo_analysis_lib import Vector, Team, Class, ClassList

class Kill:
    @property
    def dead_id(self) -> uint16:
        """User ID of the killed / dead player."""
    @property
    def dead_entity(self) -> uint32:
        """Entity ID of the killed / dead player."""
    @property
    def attacker_id(self) -> uint16:
        """User ID of the player was responsible for the final blow.
        May not be a valid ID if the player died to the environment."""
    @property
    def inflictor_id(self) -> uint32:
        """Entity ID of the thing responsible for the final blow. May not be a player."""
    @property
    def weapon(self) -> str: ...
    @property
    def assister(self) -> uint16 | None:
        """User ID of the assister. Could be none."""
    @property
    def dead_rocketjumping(self) -> bool:
        """If the killed/dead player was rocket jumping when they died."""
    @property
    def tick(self) -> uint32: ...