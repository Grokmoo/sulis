function on_activate(parent, ability)
  ability:activate(parent)

  local effect = parent:create_effect(ability:name(), ability:duration())
  effect:set_tag("haste")
  
  local stats = parent:stats()
  local amount = (2 + stats.intellect_bonus / 20) * game:ap_display_factor()
  effect:add_num_bonus("ap", amount)
  
  local gen = parent:create_anim("haste")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-0.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  effect:add_anim(gen)
  effect:apply()
  
  game:play_sfx("sfx/short_wind_sound")
end
