function on_activate(parent, ability)
  targets = parent:targets():hostile():visible()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()
  
  speed = 10.0
  dist = parent:dist_to_entity(target)
  duration = 0.5 + dist / speed
  vx = (target:x() - parent:x()) / duration
  vy = (target:y() - parent:y()) / duration
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_anim_update_fn("attack_target")
  
  gen = parent:create_particle_generator("fire_particle", duration)
  gen:set_position(gen:param(parent:x(), vx), gen:param(parent:y(), vy))
  gen:set_gen_rate(gen:param(70.0))
  gen:set_initial_gen(35.0)
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-vx / 5.0, 0.0)),
    gen:dist_param(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-vy / 5.0, 0.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:add_callback(cb, duration - 0.1)
  gen:activate()
  
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  target = targets:first()
  parent:special_attack(target, "Reflex", 20, 30, "Fire")
  
  gen = target:create_particle_generator("fire_particle", 0.6)
  gen:set_initial_gen(50.0)
  gen:set_position(gen:param(target:x()), gen:param(target:y()))
  gen:set_particle_size_dist(gen:fixed_dist(0.3), gen:fixed_dist(0.3))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-5.0, 5.0)),
    gen:dist_param(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-5.0, 5.0), gen:fixed_dist(5.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:activate()
end
