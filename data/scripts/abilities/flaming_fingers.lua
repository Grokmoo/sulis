max_dist = 10

function on_activate(parent, ability)
  targets = parent:targets():without_self()
  
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(max_dist * 2)
  targeter:set_shape_cone(parent:center_x(), parent:center_y(), max_dist, math.pi / 3) 
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  anim = parent:wait_anim(0.3)
  cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("create_fire_surface")
  anim:set_completion_callback(cb)
  anim:activate()

  pos = targets:selected_point()
  
  delta_x = pos.x - parent:center_x()
  delta_y = pos.y - parent:center_y()
  angle = game:atan2(delta_x, delta_y)
  
  duration = 1.5
  
  gen = parent:create_particle_generator("fire_particle", duration)
  gen:set_position(gen:param(parent:center_x()), gen:param(parent:center_y()))
  gen:set_gen_rate(gen:param(500.0, -500))
  gen:set_initial_gen(500.0)
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  gen:set_particle_position_dist(
    gen:dist_param(gen:uniform_dist(-0.1, 0.1),
    gen:angular_dist(angle - math.pi / 6, angle + math.pi / 6, 0, 20)))
    
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  
  targets_table = targets:to_table()
  for i = 1, #targets_table do
    dist = parent:dist_to_entity(targets_table[i])
    cb_dur = duration * dist / max_dist
    
    cb = ability:create_callback(parent)
	cb:add_target(targets_table[i])
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, cb_dur)
  end
  
  gen:activate()
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  target = targets:first()

  if target:is_valid() then
    parent:special_attack(target, "Reflex", "Spell", 20, 30, 0, "Fire")
  end
end

function create_fire_surface(parent, ability, targets)
  points = targets:random_affected_points(0.3)
  surf = parent:create_surface(ability:name(), points, 1)
  surf:set_squares_to_fire_on_moved(3)
  
  cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("on_round_elapsed")
  cb:set_on_moved_in_surface_fn("on_moved")
  surf:add_callback(cb)
  
  gen = parent:create_particle_generator("fire_particle")
  gen:set_alpha(gen:param(0.75))
  gen:set_gen_rate(gen:param(30.0))
  gen:set_position(gen:param(0.0), gen:param(0.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-0.1, 0.1)),
								 gen:dist_param(gen:uniform_dist(0.0, 0.5), gen:uniform_dist(-2.0, -3.0)))
  gen:set_draw_above_entities()
  surf:add_anim(gen)
  
  below = parent:create_anim("particles/circle16")
  below:set_draw_below_entities()
  below:set_position(below:param(-0.25), below:param(-0.25))
  below:set_particle_size_dist(below:fixed_dist(1.5), below:fixed_dist(1.5))
  below:set_color(below:param(0.8), below:param(0.5), below:param(0.0), below:param(0.2))
  surf:add_anim(below)
  
  surf:apply()
end

function on_moved(parent, ability, targets)
  target = targets:first()
  target:take_damage(2, 4, "Fire", 5)
end

function on_round_elapsed(parent, ability, targets)
  targets = targets:to_table()
  for i = 1, #targets do
	targets[i]:take_damage(2, 4, "Fire", 5)
  end
end
