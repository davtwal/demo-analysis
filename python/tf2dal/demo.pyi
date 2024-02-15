"""
Contains the structures for DemoData and TIckData.

DemoData contains one of the following:

    1) Information on the entire demo file. If this is the case, 
        `DemoData.rounds` will contain multiple rounds inside of it and may 
        contain ticks that are not a part of an actual round of gameplay.
    
    OR

    2) Information on a single round in the demo. In this case, `rounds` will
        only contain one round. The map name, demo file name, duration, and 
        player reach bounds will be the same as the whole demo. Note that the
        duration is NOT the duration of the round in time in this case!
"""

print("| Importing dal.demo...")

from typing import List, Dict
from numpy  import float32

from .         import EntityID, UserID, DemoTick
from .game     import Round, World
from .entities import Player, Sentry, Dispenser, Teleporter, Medigun
from .events   import Kill

class TickData:
    """Contains information about the state of a tick."""
    @property
    def players(self) -> List[Player]:
        """List of all players that have ever been seen up until this tick."""

    @property
    def sentries(self) -> Dict[Sentry]: ...

    @property
    def dispensers(self) -> Dict[Dispenser]: ...

    @property
    def teleporters(self) -> Dict[Teleporter]: ...

    @property
    def mediguns(self) -> List[Medigun]:
        """List of all mediguns that have ever been seen up until this tick."""

    @property
    def tick(self) -> DemoTick:
        """Index of the tick."""

    @property
    def tick_delta(self) -> float32:
        """Amount of actual seconds this tick lasted."""

    def get_player_by_entityid(self, entity_id: EntityID) -> Player | None: ...
    def get_player_by_userid(self, user_id: UserID) -> Player | None: ...

class DemoData:
    """Contains either information for the whole demo,
    or information on a piece of the demo that contains multiple
    tick states (e.g. a round)."""
    @property
    def demo_filename(self) -> str: ...

    @property
    def map_name(self) -> str: ...

    @property
    def duration(self) -> float32: 
        """The duration of the demo file in seconds.
        If this DemoData is a single round of a demo file, then this
        will not change! It will still be the duration of the entire demo,
        not the duration of the round."""
        ...

    @property
    def rounds(self) -> List[Round]: ...

    @property
    def kills(self) -> List[Kill]: ...

    @property
    def player_reach_bounds(self) -> World: ...

    @property
    def tick_states(self) -> Dict[DemoTick, TickData]: ...

    def round_data(self, round: Round) -> DemoData:
        """Limit the data to a specific round.
        This DOES copy. Don't use this function if `len(rounds) < 2` !"""