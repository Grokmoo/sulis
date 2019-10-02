function on_activate(parent, ability)
  local targets = parent:targets():friendly():without_self():visible_within(8)
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(8.0)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  local stats = target:stats() -- this spell uses the target's will
  local amount = stats.will / 8
  
  local effect = target:create_effect(ability:name(), ability:duration())
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_round_elapsed_fn("apply_heal")
  
  effect:add_num_bonus("armor", amount)
  effect:add_callback(cb)
  
  local anim = target:create_particle_generator("sparkle")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.7), anim:fixed_dist(0.7))
  anim:set_gen_rate(anim:param(5.0))
  anim:set_initial_gen(4.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-1.0, 1.0)),
                                  anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-1.5, 1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  effect:add_anim(anim)
  effect:apply()
  
  ability:activate(parent)
end

function apply_heal(parent, ability, targets)
  local stats = parent:stats()
  local target = targets:first()
  
  target:heal_damage(10 + stats.caster_level / 3 + stats.wisdom_bonus / 3)
end
