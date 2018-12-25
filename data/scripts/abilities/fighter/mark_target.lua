function on_activate(parent, ability)
  targets = parent:targets():hostile():visible()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  stats = parent:stats()
  ability:activate(parent)

  target = targets:first()
  
  hit = parent:special_attack(target, "Fortitude", "Melee")
  amount = 25 + stats.strength_bonus + stats.level
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    amount = amount / 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    amount = amount * 1.5
  end
  
  effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("vulnerable")
  effect:add_resistance(-amount, "Piercing")
  effect:add_resistance(-amount, "Slashing")
  effect:add_resistance(-amount, "Crushing")
  
  gen = target:create_particle_generator("arrow_down")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(0.0), gen:param(-0.5))
  gen:set_particle_size_dist(gen:fixed_dist(0.5), gen:fixed_dist(0.5))
  gen:set_gen_rate(gen:param(6.0))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.7, 0.7), gen:uniform_dist(-0.1, 0.1)),
                                 gen:dist_param(gen:fixed_dist(0.0), gen:uniform_dist(1.0, 1.5)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.75))
  gen:set_color(gen:param(1.0), gen:param(1.0), gen:param(0.0))
  effect:add_anim(gen)
  effect:apply()
end
