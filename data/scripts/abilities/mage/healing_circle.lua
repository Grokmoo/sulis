function on_activate(parent, ability)
  local targets = parent:targets()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(10.0)
  targeter:set_free_select(10.0)
  -- targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_object_size("9by9round")
  targeter:add_all_effectable(targets)
  targeter:allow_affected_points_impass(false)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local points = targets:affected_points()
  local surface = parent:create_surface(ability:name(), points, ability:duration())
  
  local cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("apply_heal")
  surface:add_callback(cb)
  
  local anim = parent:create_particle_generator("heal")
  anim:set_position(anim:param(0.0), anim:param(0.0))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  anim:set_gen_rate(anim:param(3.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.5, -1.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  anim:set_particle_frame_time_offset_dist(anim:uniform_dist(0.0, 0.8))
  anim:set_draw_above_entities()
  surface:add_anim(anim)
  
  local below = parent:create_anim("particles/circle16")
  below:set_draw_below_entities()
  below:set_position(below:param(-0.5), below:param(-0.5))
  below:set_particle_size_dist(below:fixed_dist(2.0), below:fixed_dist(2.0))
  below:set_color(below:param(0.0), below:param(1.0), below:param(1.0), below:param(0.1))
  surface:add_anim(below)
  
  surface:apply()
  ability:activate(parent)
  
  local stats = parent:stats()
  local amount = 20 + stats.caster_level + stats.intellect_bonus / 2
  
  local targets = targets:friendly():to_table()
  for i = 1, #targets do
	targets[i]:heal_damage(amount)
  end
end

function apply_heal(parent, ability, targets)
  local targets = targets:friendly()
  --points = targets:affected_points()
  --for i = 1, #points do
  --  point = points[i]
  --   game:log("point " .. point.x .. ", " .. point.y)
  --end
  
  local stats = parent:stats()
  local amount = 20 + stats.caster_level + stats.intellect_bonus / 2

  local targets = targets:to_table()
  for i = 1, #targets do
	targets[i]:heal_damage(amount)
  end
end
