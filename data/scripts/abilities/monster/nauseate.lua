function on_activate(parent, ability)
  targets = parent:targets():hostile():reachable()
  
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
  amount = -4
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
  effect:set_tag("nauseate")
  effect:add_attribute_bonus("Strength", amount)
  effect:add_attribute_bonus("Dexterity", amount)
  effect:add_attribute_bonus("Endurance", amount)
  
  anim = target:create_particle_generator("sparkle")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.5), anim:fixed_dist(0.5))
  anim:set_gen_rate(anim:param(6.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.0, -1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(1.2))
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(0.1), anim:param(0.5))
  effect:add_anim(anim)
  effect:apply()
end
