function on_activate(parent, ability)
  local effect = parent:create_effect(ability:name(), ability:duration())
  effect:set_tag("magic_defense")
  
  local stats = parent:stats()
  local amount = 40 + stats.caster_level + stats.wisdom_bonus
  effect:add_resistance(amount, "Fire")
  effect:add_resistance(amount, "Cold")
  effect:add_resistance(amount, "Acid")
  effect:add_resistance(amount, "Shock")

  local gen = parent:create_particle_generator("wind_particle", duration)
  setup_anim(gen)
  effect:add_anim(gen)
  
  local gen = parent:create_particle_generator("wind_particle", duration)
  gen:set_color(gen:param(0.0), gen:param(0.2), gen:param(1.0))
  setup_anim(gen)
  effect:add_anim(gen)
  
  local gen = parent:create_particle_generator("wind_particle", duration)
  gen:set_color(gen:param(0.0), gen:param(1.0), gen:param(0.0))
  setup_anim(gen)
  effect:add_anim(gen)
  
  local gen = parent:create_particle_generator("fire_particle")
  setup_anim(gen)
  effect:add_anim(gen)
  
  effect:apply()
  ability:activate(parent)
end

function setup_anim(gen)
  gen:set_moves_with_parent()
  gen:set_gen_rate(gen:param(20.0))
  gen:set_position(gen:param(-0.25), gen:param(0.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-1.0, 1.0)),
    gen:dist_param(gen:uniform_dist(0.0, 0.2), gen:uniform_dist(-1.0, 0.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
end
