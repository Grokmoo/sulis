function on_activate(parent, item)
  effect = parent:create_effect(item:name(), item:duration())

  cb = item:create_callback(parent)
  cb:set_after_attack_fn("apply_poison")
  effect:add_callback(cb)
  
  anim = parent:create_particle_generator("particles/circle4")
  anim:set_moves_with_parent()
  anim:set_initial_gen(8.0)
  anim:set_color(anim:param(1.0), anim:param(1.0), anim:param(0.0))
  anim:set_gen_rate(anim:param(15.0))
  anim:set_position(anim:param(-1.0), anim:param(-1.0))
  anim:set_particle_size_dist(anim:fixed_dist(0.3), anim:fixed_dist(0.3))
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.2, 0.2), anim:uniform_dist(-1.0, 1.0)),
    anim:dist_param(anim:uniform_dist(-0.2, 0.2), anim:uniform_dist(-1.0, 1.0), anim:fixed_dist(5.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.3))
  effect:add_anim(anim)
  effect:apply()
  
  item:activate(parent)
end

function apply_poison(parent, item, targets, hit)
  target = targets:first()

  if hit:is_miss() then return end
  
  duration = 2
  
  if hit:is_graze() then
    duration = duration - 1
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = duration + 1
  end
  
  effect = target:create_effect(item:name(), duration)
  
  cb = item:create_callback(parent)
  cb:add_target(target)
  cb:set_on_round_elapsed_fn("poison_round_elapsed")
  effect:add_callback(cb)
  
  anim = target:create_particle_generator("particles/circle8")
  anim:set_moves_with_parent()
  anim:set_initial_gen(8.0)
  anim:set_color(anim:param(1.0), anim:param(1.0), anim:param(0.0))
  anim:set_gen_rate(anim:param(15.0))
  anim:set_position(anim:param(0.0), anim:param(-1.0))
  anim:set_particle_size_dist(anim:fixed_dist(0.5), anim:fixed_dist(0.5))
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.5, 0.5), anim:uniform_dist(-1.0, 1.0)),
    anim:dist_param(anim:uniform_dist(-0.5, 0.5), anim:uniform_dist(-1.0, 1.0), anim:fixed_dist(5.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.3))
  effect:add_anim(anim)
  
  effect:apply()
end

function poison_round_elapsed(parent, item, targets)
  target = targets:first()

  target:take_damage(2, 4, "Raw")
end
