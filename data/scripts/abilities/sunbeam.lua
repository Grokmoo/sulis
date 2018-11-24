function on_activate(parent, ability)
  targets = parent:targets()
  
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(15.0)
  targeter:set_shape_object_size("9by9round")
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  position = targets:selected_point()

  anim = parent:create_particle_generator("particles/circle20", 1.0)
  anim:set_position(anim:param(position.x - 1.0), anim:param(position.y - 10.0))
  anim:set_particle_size_dist(anim:fixed_dist(2.5), anim:fixed_dist(2.5))
  anim:set_gen_rate(anim:param(0.0))
  anim:set_initial_gen(400.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-1.5, 1.5), anim:fixed_dist(0.0)),
                                  anim:dist_param(anim:uniform_dist(0.0, 10.0), anim:fixed_dist(0.0)))
  anim:set_color(anim:param(1.0), anim:param(1.0), anim:param(1.0), anim:param(0.0, 1.0, -1.0))
  anim:set_particle_duration_dist(anim:fixed_dist(1.0))
  anim:activate()
  
  anim = parent:create_particle_generator("particles/circle20", 2.0)
  anim:set_position(anim:param(position.x - 1.0), anim:param(position.y - 1.0))
  anim:set_particle_size_dist(anim:fixed_dist(2.5), anim:fixed_dist(2.5))
  anim:set_gen_rate(anim:param(0.0))
  anim:set_initial_gen(500.0)
  anim:set_particle_position_dist(anim:dist_param(anim:angular_dist(0.0, 2 * math.pi, 0.0, 4.0)))
  anim:set_color(anim:param(1.0), anim:param(1.0), anim:param(1.0), anim:param(0.0, 0.5, -0.5))
  anim:set_particle_duration_dist(anim:fixed_dist(2.0))
  anim:activate()
  
  targets = targets:to_table()
  for i = 1, #targets do
    attack_target(parent, ability, targets[i])
  end
  
  
  ability:activate(parent)
end

function attack_target(parent, ability, target)
  stats = parent:stats()
  min_dmg = 5 + stats.caster_level / 4 + stats.wisdom_bonus / 4
  max_dmg = 10 + stats.wisdom_bonus / 2 + stats.caster_level / 2
  hit = parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 6, "Fire")
  
  amount = -(20 + stats.wisdom_bonus)
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    amount = amount / 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    amount = amount * 1.5
  end
  
  game:log("attack " .. target:name())
  
  effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("blind")
  effect:add_num_bonus("melee_accuracy", amount)
  effect:add_num_bonus("ranged_accuracy", amount)
  effect:add_num_bonus("spell_accuracy", amount)
  
  anim = target:create_particle_generator("particles/circle4")
  anim:set_moves_with_parent()
  anim:set_color(anim:param(0.0), anim:param(0.0), anim:param(0.0))
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.5), anim:fixed_dist(0.5))
  anim:set_gen_rate(anim:param(6.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.5, 0.5), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.0, -1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(1.0))
  effect:add_anim(anim)
  effect:apply()
end

