function on_activate(parent, ability)
  targets = parent:targets():hostile():visible()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()
  
  speed = 10.0
  dist = parent:dist(target)
  duration = dist / speed
  vx = (target:x() - parent:x()) / duration
  vy = (target:y() - parent:y()) / duration
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:register_fn("on_anim_complete")
  
  gen = parent:create_particle_generator("particles/circle8", duration)
  gen:set_position(gen:speed_param(parent:x(), vx), gen:speed_param(parent:y(), vy))
  gen:set_gen_rate(gen:fixed_param(50.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_x_dist(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-vx / 8.0, 0.0), gen:zero_dist())
  gen:set_particle_y_dist(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-vy / 8.0, 0.0), gen:zero_dist())
  gen:set_particle_duration_dist(gen:fixed_dist(0.5))
  gen:set_callback(cb)
  gen:activate()
end

function on_anim_complete(parent, ability, targets)
  target = targets:first()
  parent:special_attack(target, "Reflex", 10, 20, "Fire")
end
