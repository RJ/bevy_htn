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


## Hmm. Efects, Expected Effects, and ECS-modified GameState

not sure if i need expected effects. 
effects presumably are applied to GameState once an operator successfully completes.
do we do it that way?
still need ECS system to update GameState when external shit happens.
maybe just ECS systems update GameState, and all effects in the htn are expected during planning?

could that lead to problems where the ECS systems don't update the gamestate in the way described by the effects?


In the literature, HTN tasks have effects and expected_effects. Both are applied to the transient copy of the planner state used during the planning process, but only effects are permanently applied to the planner state once a task completes successfully.

Meanwhile, external factors can change the planner state. Sensors for the character detect changes and update the planner state as needed, even when no plan is executing.

So when we execute a task from the planner, the sensors should result in changes to the planner state 

https://github.com/makspll/bevy_mod_scripting/blob/a4d1ffbcae98f42393ab447d73efe9b0b543426f/crates/bevy_mod_scripting_core/src/bindings/world.rs#L642

## TODO

HtpOperator to_tree doesn't need to return an Option if we always use it.
do we still need the htn trigger alternative? that can provide the planned task id, which is
maybe useful... but can you just get that by querying the Plan component via ctx.supervisor_entity()?
maybe we can just insert a context component so behaviours can look it up if needed?

while a plan is running, if gamestate changes, verify all preconditions of current and remaining tasks in the plan still pass,
and if not, fail the plan.

## HTN bugs

troll does 3 attacks, trunk health goes to 0
starts to move towards trunk to uproot it
enemy goes out of range, replans, no enemy in range so replans to nav to last enemy pos, roar.
this causes enemy to become in range (via sensor updating gamestate), goes back to uproot plan.
bounces between two plans never getting anywhere, moving just in and out of enemy range.

need to implement MTR.

