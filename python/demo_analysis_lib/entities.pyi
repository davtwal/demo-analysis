from numpy import uint32, uint16, float32, uint8
from enum import Enum
#from typing import List, Dict

from demo_analysis_lib import Vector, Team, Class, ClassList

class World:
    """Defines the boundaries of the world as given by the demofile.
    You can expect bound_min.x < bound_max.x, and so on."""
    @property
    def bound_min(self) -> Vector: ...

    @property
    def bound_max(self) -> Vector: ...

class UserInfo:
    @property
    def classes(self) -> ClassList: ...

    @property
    def name(self) -> str: ...

    @property
    def user_id(self) -> uint16: ...

    @property
    def steam_id(self) -> str: ...

    @property
    def entity_id(self) -> uint32: ...

    @property
    def team(self) -> Team: ...

class PlayerState(Enum):
    Alive = uint8(0)
    Dying = uint8(1)
    Death = uint8(2)
    Respawnable = uint8(3)

class Player:
    """All properties of players that are tracked."""
    @property
    def position(self) -> Vector: ...
    @property
    def health(self) -> uint16: ...
    @property
    def max_health(self) -> uint16: ...
    @property
    def player_class(self) -> Class: ...
    @property
    def team(self) -> Team:...
    @property
    def view_angle(self) -> float32:
        """View angle is the angle the player is looking. 0 = positive x"""
    @property
    def pitch_angle(self) -> float32:
        """The angle the player is looking up and down."""
    @property
    def state(self) -> PlayerState: ...
    @property
    def info(self) -> UserInfo | None: ...
    @property
    def charge(self) -> uint8: ...
    @property
    def simtime(self) -> uint16: ...
    @property
    def ping(self) -> uint16: ...
    @property
    def in_pvs(self) -> bool: ...

    def distance_from(self, other: Player) -> float32: ...
    def distance_from_xy(self, other: Player) -> float32:
        """Distance, but only taking into account X and Y axes."""
    def height_diff(self, other: Player) -> float32:
        """Height difference. Negative means self is below other."""
    def is_alive(self) -> bool: ...

# todo
class Sentry: ...

class Dispenser: ...

class Teleporter: ...