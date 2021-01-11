function on_activate(parent, item)
  local targets = parent:targets():friendly():visible_within(5.0)
  
  local targeter = parent:create_targeter_for_item(item)
  targeter:set_selection_radius(5.0)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, item, targets)
  local target = targets:first()
  
  local effect = target:create_effect(item:name(), item:duration())
  
  local cb = item:create_callback(parent)
  cb:add_target(target)
  cb:set_on_round_elapsed_fn("apply_heal")
  effect:add_callback(cb)
  
  local anim = target:create_particle_generator("heal")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.75), anim:fixed_dist(0.75))
  anim:set_gen_rate(anim:param(2.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.5, -1.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  effect:add_anim(anim)
  effect:apply()
  
  item:activate(parent)
  
  -- remove one disease or injury
  local injuries = target:get_effects_with_tag("injury")
  for _, effect in ipairs(injuries) do
    effect:mark_for_removal()
	return
  end
  
  local diseases = target:get_effects_with_tag("disease")
  for _, effect in ipairs(diseases) do
    effect:mark_for_removal()
	return
  end
end

function apply_heal(parent, item, targets)
  local target = targets:first()
  
  target:heal_damage(4)
end
