function on_activate(parent, ability)
  if not parent:inventory():has_equipped_shield() then
    game:say_line("You must have a shield equipped.", parent)
    return
  end
  
  if parent:has_active_mode() then
    game:say_line("Only one mode may be active at a time.", parent)
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

  if parent:ability_level(ability) > 1 then
    cb = ability:create_callback(parent)
    cb:set_after_defense_fn("after_defense")
    effect:add_callback(cb)
  end

  gen = parent:create_anim("shield")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-2.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  effect:add_anim(gen)
  effect:apply()

  ability:activate(parent)
end

function after_defense(parent, ability, targets, hit)
  if hit:total_damage() < 2 then return end
  
  target = targets:first()

  if parent:dist_to_entity(target) > 4.0 then
    return
  end

  max_damage = math.floor(hit:total_damage() * 0.5)
  
  target:take_damage(max_damage, max_damage, "Raw")
end
