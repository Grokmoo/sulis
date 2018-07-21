function on_activate(parent, ability)
  if not parent:stats().attack_is_ranged then
    game:say_line("You must have a ranged weapon equipped.", parent)
    return
  end
  
  if parent:has_active_mode() then
    game:say_line("Only one mode may be active at a time.", parent)
    return
  end

  effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  effect:add_num_bonus("attack_cost", 1000)
  effect:add_damage(0, 0, 5)

  if parent:ability_level(ability) > 1 then
	effect:add_num_bonus("accuracy", 25)
	effect:add_num_bonus("crit_multiplier", 1.0)
  else
    effect:add_num_bonus("accuracy", 15)
	effect:add_num_bonus("crit_multiplier", 0.50)
  end
  
  gen = parent:create_anim("precise_arrow")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-2.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  effect:add_anim(gen)
  effect:apply()

  ability:activate(parent)
end
