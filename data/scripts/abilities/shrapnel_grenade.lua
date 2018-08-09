frag_radius = 4.0

function on_activate(parent, ability)
  targets = parent:targets()
  
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(8.0)
  -- targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_object_size("7by7round")
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  selected_point = targets:selected_point()
  speed = 20.0
  dist = parent:dist_to_point(selected_point)
  duration = dist / speed
  vx = (selected_point.x - parent:center_x()) / duration
  vy = (selected_point.y - parent:center_y()) / duration
  
  cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("create_explosion")
  
  gen = parent:create_anim("particles/circle12", duration)
  gen:set_color(gen:param(0.5), gen:param(0.5), gen:param(0.5))
  gen:set_position(gen:param(parent:center_x(), vx), gen:param(parent:center_y(), vy))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  gen:set_particle_position_dist(gen:dist_param(gen:fixed_dist(0.0), gen:fixed_dist(-vx / 5.0)),
    gen:dist_param(gen:fixed_dist(0.0), gen:fixed_dist(-vy / 5.0)))
  gen:set_completion_callback(cb)
  gen:activate()
  
  ability:activate(parent)
end

function create_explosion(parent, ability, targets)
  duration = 1.2
  
  position = targets:selected_point()
  
  gen = parent:create_particle_generator("particles/circle4", duration)
  gen:set_initial_gen(100.0)
  gen:set_gen_rate(gen:param(20.0, 0, -500, -500))
  gen:set_position(gen:param(position.x), gen:param(position.y))
  gen:set_particle_size_dist(gen:fixed_dist(0.3), gen:fixed_dist(0.3))
  speed = 1.5 * frag_radius / 0.6
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1), gen:angular_dist(0.0, 2 * math.pi, 0, speed)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:set_color(gen:param(0.5), gen:param(0.5), gen:param(0.5))
  
  targets = targets:to_table()
  for i = 1, #targets do
    cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, targets[i]:dist_to_point(position) / speed)
  end
  
  gen:activate()
end

function attack_target(parent, ability, targets)
  target = targets:first()

  if target:is_valid() then
    target = targets:first()
    parent:special_attack(target, "Reflex", "Ranged", 20, 30, 10, "Piercing")
  end
end
