from numpy import uint32, uint16, float32, uint8
from typing import List, Dict
from enum import Enum

from demo_analysis_lib import Team, Vector
from demo_analysis_lib.entities import Player

class TickPlayerData:
    """Analysis data for a single player during a single tick."""
    @property
    def entity_id(self) -> uint32: ...
    @property
    def user_id(self) -> uint16: ...
    @property
    def dist_from_medic(self) -> float32:
        """Distance in XYZ from the medic on the team.
        If the the player IS the medic the distance is ~0,
        if the medic is dead, or the team has no medic, distance is -1."""
    @property
    def dist_from_team_avg(self) -> float32:
        """Distance from the team average position XYZ.
        If the player is the only person on the team, distance is ~0."""
    @property
    def dist_from_group_avg(self) -> float32:
        """Distance from the group average position XYZ.
        If the player is the only person on the team, distance is ~0."""

class GroupingType(Enum):
    NONE = 0 #Invalid group
    ISOLATED = 1 #Only one player in group
    ISOLATED_COMBO = 2 #Contains medic and only one other player
    COMBO = 3 #Contains medic and at least 2 other players
    FLANK = 4 #Contains at least 2 players but no medic

class TickPlayerGrouping:
    @property
    def group_type(self) -> GroupingType : ...
    @property
    def players(self) -> List[Player]: ...
    @property
    def avg_pos(self) -> Vector: ...

class TickTeamAnalysis:
    @property
    def team(self) -> Team: ...
    @property
    def medic(self) -> Player | None:
        """If the team has no medic or their medic is dead,
        returns None."""