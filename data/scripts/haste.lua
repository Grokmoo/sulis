function on_activate(parent, ability)
  targets = parent:targets():friendly():visible()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  target = targets:first()
  
  effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("haste")
  effect:add_num_bonus("ap", 2000)
  
  gen = target:create_anim("haste")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-0.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  effect:apply(gen)
end
