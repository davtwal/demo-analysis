"""
Contains definitions for each entity.
An entity is (pretty much) any game object, including players, weapons, hats, 
objectives, and many more.
"""

from numpy import uint16, float32, uint8
from enum import Enum

from .math      import Vector
from .game      import Team, Class, ClassList
from .          import EntityID, UserID, DemoTick

class UserInfo:
    @property
    def classes(self) -> ClassList: ...

    @property
    def name(self) -> str: ...

    @property
    def user_id(self) -> UserID: ...

    @property
    def steam_id(self) -> str: ...

    @property
    def entity_id(self) -> EntityID: ...

    @property
    def team(self) -> Team: 
        """Same as Player.team"""

class PlayerState(Enum):
    """Life state of a player. Just use Player.is_alive."""
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
    def player_class(self) -> Class: 
        """What class they're currently playing."""
    @property
    def team(self) -> Team: 
        """The team the player is on. Can be any of:
         
        - Other (0): Used for non-player entities.
        - Spectator (1)
        - Red (2)
        - Blue (3)"""

    @property
    def view_angle(self) -> float32:
        """Part of the direction that the player is looking.
        This specific measure is the angle from the positive x
        axis, moving counter-clockwise, that the player is facing.
        Should range from 0 to 2pi, or maybe -pi to pi."""

    @property
    def pitch_angle(self) -> float32:
        """The angle the player is looking up and down.
        I believe this is measured with 0 looking neither up or down,
        and ranging from -pi/2 to pi/2. Could also range from 0 to pi."""
    @property
    def state(self) -> PlayerState: 
        """Just use is_alive unless you really need to know"""

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

    @property
    def time_since_last_hurt(self) -> float32: ...
    # todo: properties

    def distance_from(self, other: Player) -> float32: ...
    def distance_from_xy(self, other: Player) -> float32:
        """Distance, but only taking into account X and Y axes."""
        ...
    def height_diff(self, other: Player) -> float32:
        """Height difference between two players.
        Negative means self is below other."""

    def is_alive(self) -> bool: ...

class Medigun:
    @property
    def owner(self) -> EntityID:
        """The medigun owner's (wielder's) entity ID."""
        ...

    @property
    def charge(self) -> float32:
        """Ubercharge %."""
        ...
    
    @property
    def heal_target(self) -> EntityID:
        """The EntityID of the person being healed."""
        ...

    @property
    def is_healing(self) -> bool:
        """If the medic is actually healing"""
        ...

    @property
    def is_holstered(self) -> bool:
        """If the medigun is currently put away"""
        ...

# todo
class Sentry: ...

class Dispenser: ...

class Teleporter: ...