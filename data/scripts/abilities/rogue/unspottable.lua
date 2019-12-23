function on_activate(parent, ability)
  local hide = parent:get_ability("hide")
  local cb = hide:create_callback(parent)
  cb:set_on_anim_complete_fn("activate_no_ap")
  local hide_anim = parent:wait_anim(0.3)
  hide_anim:set_completion_callback(cb)
  hide_anim:activate()
  
  local effect = parent:create_effect(ability:name(), ability:duration() + parent:ability_level(ability) - 1)
  effect:set_tag("unspottable")

  local gen = parent:create_particle_generator("wind_particle", duration)
  gen:set_moves_with_parent()
  gen:set_gen_rate(gen:param(70.0))
  gen:set_position(gen:param(-0.5), gen:param(-0.5))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-1.0, 1.0), gen:uniform_dist(-1.0, 1.0)),
    gen:dist_param(gen:uniform_dist(-1.0, 1.0), gen:uniform_dist(-1.0, 0.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:set_color(gen:param(1.0), gen:param(0.0), gen:param(0.0))
  effect:add_anim(gen)
  effect:apply()
  
  ability:activate(parent)
end
