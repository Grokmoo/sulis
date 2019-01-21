function on_activate(parent, ability)
  local targets = parent:targets()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_free_select(16.0)
  targeter:set_shape_circle(5.0)
  targeter:add_all_effectable(targets)
  targeter:allow_affected_points_impass(false)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local points = targets:affected_points()
  local surface = parent:create_surface(ability:name(), points, ability:duration())
  
  surface:set_squares_to_fire_on_moved(6)
  local cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("on_round_elapsed")
  cb:set_on_moved_in_surface_fn("on_moved")
  surface:add_callback(cb)
  
  local gen = parent:create_particle_generator("hail")
  gen:set_initial_gen(2.0)
  gen:set_gen_rate(gen:param(5.0))
  gen:set_position(gen:param(0.0), gen:param(-4.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.3), gen:fixed_dist(0.6))
  gen:set_particle_duration_dist(gen:fixed_dist(0.79))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-0.5, 0.5)),
								 gen:dist_param(gen:uniform_dist(0.0, 0.0), gen:uniform_dist(3.5, 4.5)))
  gen:set_particle_frame_time_offset_dist(gen:uniform_dist(0.0, 0.1))
  gen:set_draw_above_entities()

  local anim = parent:wait_anim(0.2)
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("do_attack")
  anim:set_completion_callback(cb)
  anim:activate()
  
  surface:add_anim(gen)
  surface:apply()
  ability:activate(parent)
end

function on_moved(parent, ability, targets)
  do_attack(parent, ability, targets)
end

function on_round_elapsed(parent, ability, targets)
  do_attack(parent, ability, targets)
end

function do_attack(parent, ability, targets)
  local targets = targets:to_table()
  for i = 1, #targets do
    local target = targets[i]
    target:take_damage(parent, 3, 6, "Cold", 4)
    target:take_damage(parent, 3, 6, "Piercing", 4)
  end
end