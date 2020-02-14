function on_activate(parent, ability)
  local targets = parent:targets()
  
  local smoke_radius = 5.0
  if parent:has_ability("mechanical_mastery") then
    smoke_radius = smoke_radius + 2.0
  end
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:set_free_select(ability:range())
  -- targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_circle(smoke_radius)
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local selected_point = targets:selected_point()
  local speed = 15.0
  local dist = parent:dist_to_point(selected_point)
  local duration = dist / speed
  local vx = (selected_point.x - parent:center_x()) / duration
  local vy = (selected_point.y - parent:center_y()) / duration
  
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("create_smoke")
  
  local gen = parent:create_anim("particles/circle12", duration)
  gen:set_color(gen:param(0.5), gen:param(0.5), gen:param(0.5))
  gen:set_position(gen:param(parent:center_x(), vx), gen:param(parent:center_y(), vy))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  gen:set_particle_position_dist(gen:dist_param(gen:fixed_dist(0.0), gen:fixed_dist(-vx / 5.0)),
    gen:dist_param(gen:fixed_dist(0.0), gen:fixed_dist(-vy / 5.0)))
  gen:set_completion_callback(cb)
  gen:activate()
  
  ability:activate(parent)
end

function create_smoke(parent, ability, targets)
  local points = targets:affected_points()
  local surface = parent:create_surface(ability:name(), points, ability:duration())
  surface:add_num_bonus("concealment", 15 + parent:stats().level)
  
  local s_anim = parent:create_particle_generator("particles/circle12")
  s_anim:set_position(s_anim:param(0.0), s_anim:param(0.0))
  s_anim:set_color(s_anim:param(0.5), s_anim:param(0.5), s_anim:param(0.5), s_anim:param(0.3))
  s_anim:set_gen_rate(s_anim:param(20.0))
  s_anim:set_particle_size_dist(s_anim:fixed_dist(1.0), s_anim:fixed_dist(1.0))
  s_anim:set_particle_duration_dist(s_anim:fixed_dist(1.0))
  s_anim:set_particle_position_dist(s_anim:dist_param(s_anim:uniform_dist(-1.0, 1.0), s_anim:uniform_dist(-0.2, 0.2)),
                                    s_anim:dist_param(s_anim:uniform_dist(-1.0, 1.0), s_anim:uniform_dist(-0.2, 0.2)))
  s_anim:set_draw_above_entities()
  surface:add_anim(s_anim)
  surface:apply()
end
