function on_activate(parent, ability)
  if not parent:inventory():has_equipped_shield() then
    game:say_line("You must have a shield equipped.", parent)
    return
  end
  
  if parent:has_active_mode() then
    game:say_line("Only one mode may be active at a time.", parent)
    return
  end

  stats = parent:stats()
  amount = 10 + stats.level / 2
  
  effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  effect:add_num_bonus("defense", amount)
  effect:add_num_bonus("reflex", amount / 2)
  effect:add_num_bonus("fortitude", amount / 2)
  effect:add_num_bonus("crit_threshold", 20)
  effect:add_num_bonus("crit_multiplier", -0.25)
  effect:add_num_bonus("melee_accuracy", -10)

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

  if not target:can_reach(parent) then return end

  max_damage = math.floor(hit:total_damage() * 0.3)
  
  target:take_damage(parent, max_damage, max_damage, "Raw")
end
