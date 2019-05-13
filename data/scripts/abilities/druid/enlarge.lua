function on_activate(parent, ability)
  local targets = parent:targets():friendly():visible()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_visible()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  local target = targets:first()
  
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("enlarge")
  
  effect:add_attribute_bonus("Strength", 6)
  effect:add_attribute_bonus("Dexterity", -2)
  
  effect:add_num_bonus("defense", -10)
  effect:add_num_bonus("melee_accuracy", 10)
  effect:add_num_bonus("reach", 1.0)
  
  local gen = target:create_scale_anim()
  gen:set_scale(gen:param(1.5))
  effect:add_scale_anim(gen)
  
  effect:apply()
end
