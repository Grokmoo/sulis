function fire_surface(parent, ability, points, duration)
  local surf = parent:create_surface("Fire", points, duration)
  surf:set_squares_to_fire_on_moved(3)
  
  local cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("fire_surface_on_round_elapsed")
  cb:set_on_moved_in_surface_fn("fire_surface_on_moved")
  surf:add_callback(cb)
  
  local gen = parent:create_particle_generator("fire_particle")
  gen:set_alpha(gen:param(0.75))
  gen:set_gen_rate(gen:param(20.0))
  gen:set_position(gen:param(0.0), gen:param(0.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-0.1, 0.1)),
								 gen:dist_param(gen:uniform_dist(0.0, 0.5), gen:uniform_dist(-2.0, -3.0)))
  gen:set_draw_below_entities()
  surf:add_anim(gen)
  
  local gen = parent:create_particle_generator("fire_particle")
  gen:set_alpha(gen:param(0.75))
  gen:set_gen_rate(gen:param(10.0))
  gen:set_position(gen:param(0.0), gen:param(0.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-0.1, 0.1)),
								 gen:dist_param(gen:uniform_dist(0.0, 0.5), gen:uniform_dist(-2.0, -3.0)))
  gen:set_draw_above_entities()
  surf:add_anim(gen)
  
  local below = parent:create_anim("particles/circle16")
  below:set_draw_below_entities()
  below:set_position(below:param(-0.25), below:param(-0.25))
  below:set_particle_size_dist(below:fixed_dist(1.5), below:fixed_dist(1.5))
  below:set_color(below:param(0.8), below:param(0.5), below:param(0.0), below:param(0.2))
  surf:add_anim(below)
  
  surf:apply()
end

function fire_surface_on_moved(parent, ability, targets)
  local target = targets:first()
  target:take_damage(parent, 3, 6, "Fire", 2)
end

function fire_surface_on_round_elapsed(parent, ability, targets)
  local targets = targets:to_table()
  for i = 1, #targets do
	targets[i]:take_damage(parent, 3, 6, "Fire", 2)
  end
end
