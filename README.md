Need sensors that project the ECS state onto the simplified planner state struct, which can just be a component of the NPC.

PLanning can be triggered when:
* no plan present
* plan finished
* world state changed


# behave.

Operators are Clone structs which get emitted wrapped in a trigger to execute as part of the plan.

Idea:
An operator (which must derive HtnOperator) has to convert to a bevy_behave tree.
then the task exectutor runs the bt..


## Hmm.

not sure if i need expected effects. 
effects presumably are applied to GameState once an operator successfully completes.
do we do it that way?
still need ECS system to update GameState when external shit happens.
maybe just ECS systems update GameState, and all effects in the htn are expected during planning.


time to make a gui for the example?

