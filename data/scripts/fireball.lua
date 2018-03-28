fireball_radius = 5.0
PI = 3.14159265358

function on_activate(parent, ability)
  targets = parent:targets()
  
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(15.0)
  targeter:set_circle(fireball_radius)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  selected_point = targets:selected_point()
  speed = 12.0
  dist = parent:dist_to_point(selected_point)
  duration = 0.5 + dist / speed
  vx = (selected_point.x - parent:x()) / duration
  vy = (selected_point.y - parent:y()) / duration
  
  cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_update_fn("create_explosion")
  
  gen = parent:create_particle_generator("fire_particle", duration)
  gen:set_position(gen:param(parent:x(), vx), gen:param(parent:y(), vy))
  gen:set_gen_rate(gen:param(70.0))
  gen:set_initial_gen(35.0)
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.3, 0.3), gen:uniform_dist(-vx / 5.0, 0.0)),
    gen:dist_param(gen:uniform_dist(-0.3, 0.3), gen:uniform_dist(-vy / 5.0, 0.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:add_callback(cb, duration - 0.1)
  gen:activate()
  
  ability:activate(parent)
end

function create_explosion(parent, ability, targets)
  duration = 1.2
  
  position = targets:selected_point()
  
  gen = parent:create_particle_generator("fire_particle", duration)
  gen:set_initial_gen(500.0)
  gen:set_gen_rate(gen:param(100.0, 0, -500, -500))
  gen:set_position(gen:param(position.x), gen:param(position.y))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  speed = 1.5 * fireball_radius / 0.6
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1), gen:angular_dist(0.0, 2 * PI, 0, speed)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  
  targets = targets:to_table()
  for i = 1, #targets do
    cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, targets[i]:dist_to_point(position) / speed * 1.5)
  end
  
  gen:activate()
end

function attack_target(parent, ability, targets)
  target = targets:first()

  if target:is_valid() then
    target = targets:first()
    parent:special_attack(target, "Reflex", 20, 30, "Fire")
  end
end
