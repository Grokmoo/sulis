function on_activate(parent, ability)
  targets = parent:targets():friendly():visible_within(10)
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  stats = parent:stats()
  target = targets:first()
  
  target:heal_damage(10 + stats.caster_level / 3 + stats.wisdom_bonus / 3)
  
  effect = target:create_effect(ability:name(), ability:duration())
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_round_elapsed_fn("apply_heal")
  effect:add_callback(cb)
  
  anim = target:create_particle_generator("heal")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  anim:set_gen_rate(anim:param(3.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.5, -1.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  effect:add_anim(anim)
  effect:apply()
  
  ability:activate(parent)
end

function apply_heal(parent, ability, targets)
  stats = parent:stats()
  target = targets:first()
  
  target:heal_damage(10 + stats.caster_level / 3 + stats.wisdom_bonus / 3)
end
