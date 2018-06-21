function on_activate(parent, ability)
  if not parent:inventory():has_equipped_shield() then
    return
  end
  
  if parent:has_active_mode() then
    return
  end

  effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  effect:add_num_bonus("defense", 10)
  effect:add_num_bonus("reflex", 5)
  effect:add_num_bonus("fortitude", 5)
  effect:add_num_bonus("crit_threshold", 20)
  effect:add_num_bonus("crit_multiplier", -0.25)
  effect:add_num_bonus("accuracy", -10)
  
  gen = parent:create_anim("shield")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-2.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  gen:set_color(gen:param(0.0), gen:param(0.4), gen:param(1.0))
  effect:apply(gen)

  ability:activate(parent)
end
