Need sensors that project the ECS state onto the simplified planner state struct, which can just be a component of the NPC.

PLanning can be triggered when:
* no plan present
* plan finished
* world state changed


# Plan

* Takes copy of world state.
* starts at root of domain: the first compound task in the htn file?
* finds first method to pass preconditions in the compound task called "Be an NPC".
** iter over the method's subtasks.
   if primitive: 
    check if preconditions are met, if so, apply effects to world state, and add primitive task to the plan.
   if compound:
    do same thing we did to root –



not sure if i need expected effects. 
effects presumably are applied to GameState once an operator successfully completes.
do we do it that way?
still need ECS system to update GameState when external shit happens.
maybe just ECS systems update GameState, and all effects in the htn are expected during planning.


time to make a gui for the example?

