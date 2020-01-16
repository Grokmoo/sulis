function on_activate(parent, ability)
  if parent:has_active_mode() then
    game:say_line("Only one mode may be active at a time.", parent)
    return
  end

  local effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  
  effect:add_num_bonus("ranged_accuracy", -10)

  local stats = parent:stats()
  
  if parent:ability_level(ability) > 1 then
    effect:add_num_bonus("graze_multiplier", 0.25)
	effect:add_damage(5, 12 + stats.level / 2, 0)
  else
    effect:add_damage(2, 8 + stats.level / 2, 0)
  end
  
  local cb = ability:create_callback(parent)
  cb:set_on_swap_weapons_fn("on_swap_weapons")
  effect:add_callback(cb)
  
  local gen = parent:create_anim("crossed_arrow")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-2.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  effect:add_anim(gen)
  effect:apply()

  ability:activate(parent)
end

function on_deactivate(parent, ability)
  ability:deactivate(parent)
end

function on_swap_weapons(parent, ability)
  if not parent:stats().attack_is_ranged then
    game:say_line("Powerful Shot Deactivated", parent)
    ability:deactivate(parent)
  end
end