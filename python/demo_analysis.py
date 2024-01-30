# Import the actual library.
# Note I have to add it to the path python uses to import.

# Do not change this name! Otherwise this won't work!
if __name__ == "demoanalysis":
    import os, sys
    sys.path.append(os.getcwd() + "/python/demo_analysis_lib")
    from demo_analysis_lib import entities as ent, events as evt
    import demo_analysis_lib as dal
else:
    print("This script should only be ran via demo_analysis.exe.")
    print("If developing, use the following command to analyse:")
    print("    cargo run -- -a [file path]")
    print("Optionally, add in '-n' to avoid opening the window.")
    quit()

## GROUPING
from enum import Enum

class GroupingType(Enum):
    NONE = 0        # error
    ISOLATED = 1    # 1 player
    FLANK = 3       # 2+ players, no medic
    COMBO = 2       # 2+ players, with medic

class Grouping:
    def __init__(self, players):
        self._players = players
        self._type = Grouping.type_from_player_list(players)

    @classmethod
    def type_from_player_list(playerlist) -> GroupingType:
        pass

    @property
    def type(self):
        self._type

## BASIC ANALYSIS
import numpy as np
from typing import List

class PlayerTickData:
    """Data on a player during a single tick."""
    def __init__(self, player: ent.Player, medic: ent.Player):
        self.dist_from_medic = player.distance_from_xy(medic)
        pass
    pass

class TickAnalysis:
    def __init__(self, tickdata: dal.TickData):
        self.redteam = [p for p in tickdata.players if p.team == dal.Team.Red]
        self.bluteam = [p for p in tickdata.players if p.team == dal.Team.Blu]
        # note: team may not have a medic
        self.redteam_medic = [p for p in self.redteam if p.player_class == dal.Class.Medic]
        self.bluteam_medic = [p for p in self.bluteam if p.player_class == dal.Class.Medic]

        self.red_pdata = [PlayerTickData(p, self.redteam_medic) for p in self.redteam]
        #self.blu_pdata = [PlayerTickData(p, self.)]
        
    pass

## MAIN FUNCTIONS -> DO NOT NAME CHANGE

from demo_analysis_lib import Player
from numpy import float32

# Called by the executable. DON'T CHANGE THIS FUNCTION'S NAME OR ARGUMENTS!
def demo_analysis_main(data: dal.DemoData):
    print("We got there!")
    print(data.demo_filename)

# Called by the executable. DON'T CHANGE THIS FUNCTION'S NAME OR ARGUMENTS!
def tick_analysis_main(tick: dal.TickData):
    print("Tick main")
    print(tick.tick)

# Called by the executable. DON'T CHANGE THIS FUNCTION'S NAME OR ARGUMENTS!
def generate_grouping(
    player: Player,
    teammates: List[Player],
    enemies: List[Player]
# RETURN VALUES:
# Each return value is a list of how grouped up 
) -> (List[(Player, float32)], List[(Player, float32)]) :
    pass