Need sensors that project the ECS state onto the simplified planner state struct, which can just be a component of the NPC.

PLanning can be triggered when:
* no plan present
* plan finished
* world state changed


# Plan

* Takes copy of world state.
* starts at root of domain: the first compound task in the htn file?
* finds first method to pass preconditions in the compound task called "Be an NPC".


