function on_activate(parent, ability)
  local effect = parent:create_effect(ability:name(), ability:duration())
  effect:set_tag("magic_defense")
  
  local stats = parent:stats()
  local amount = 7 + stats.caster_level / 2
  effect:add_armor_of_kind(amount, "Fire")

  local cb = ability:create_callback(parent)
  cb:set_after_defense_fn("after_defense")
  effect:add_callback(cb)

  local gen = parent:create_particle_generator("fire_particle")
  gen:set_moves_with_parent()
  gen:set_gen_rate(gen:param(70.0))
  gen:set_position(gen:param(-0.25), gen:param(0.0))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.5, 0.5), gen:uniform_dist(-1.0, 1.0)),
    gen:dist_param(gen:uniform_dist(0.0, 0.2), gen:uniform_dist(-1.0, 0.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  effect:add_anim(gen)
  effect:apply()

  ability:activate(parent)
end

function after_defense(parent, ability, targets)
  local target = targets:first()

  if parent:dist_to_entity(target) > 4.0 then
    return
  end

  local stats = parent:stats()
  local min_dmg = 4 + stats.caster_level / 4 + stats.intellect_bonus / 6
  local max_dmg = 8 + stats.caster_level / 2 + stats.intellect_bonus / 3
  parent:special_attack(target, "Reflex", "Spell", 4, 8, 0, "Fire")
end
