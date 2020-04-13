smoke_radius = 5.0

function on_activate(parent, ability)
  local targets = parent:targets()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_free_select(ability:range())
  targeter:set_selection_radius(ability:range())
  -- targeter:set_free_select_must_be_passable("1by1")
  targeter:set_shape_circle(smoke_radius)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  local points = targets:affected_points()
  local surface = parent:create_surface(ability:name(), points, ability:duration())
  surface:add_attribute_bonus("Intellect", -4)
  surface:add_attribute_bonus("Perception", -4)
  
  local cb = ability:create_callback(parent)
  cb:set_on_surface_round_elapsed_fn("on_round_elapsed")
  surface:add_callback(cb)
  
  local s_anim = parent:create_particle_generator("particles/circle12")
  s_anim:set_position(s_anim:param(0.0), s_anim:param(0.0))
  s_anim:set_color(s_anim:param(1.0), s_anim:param(1.0), s_anim:param(0.2), s_anim:param(0.3))
  s_anim:set_gen_rate(s_anim:param(20.0))
  s_anim:set_particle_size_dist(s_anim:fixed_dist(1.0), s_anim:fixed_dist(1.0))
  s_anim:set_particle_duration_dist(s_anim:fixed_dist(1.0))
  s_anim:set_particle_position_dist(s_anim:dist_param(s_anim:uniform_dist(-1.0, 1.0), s_anim:uniform_dist(-0.2, 0.2)),
                                    s_anim:dist_param(s_anim:uniform_dist(-1.0, 1.0), s_anim:uniform_dist(-0.2, 0.2)))
  s_anim:set_draw_above_entities()
  surface:add_anim(s_anim)
  surface:apply()
  
  game:play_sfx("sfx/wind2")
  game:play_sfx("sfx/freeze")
end

function on_round_elapsed(parent, ability, targets)
  
end