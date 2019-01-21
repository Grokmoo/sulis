function on_activate(parent, ability)
  local targets = parent:targets():hostile():attackable()
  
  local targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_after_attack_fn("after_attack")
  
  ability:activate(parent)
  parent:anim_weapon_attack(target, cb)
end

function after_attack(parent, ability, targets, hit)
  local target = targets:first()
  
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    factor = 1
  elseif hit:is_hit() then
    factor = 2
  elseif hit:is_crit() then
    factor = 3
  end
  
  local stats = parent:stats()
  local amount = (stats.level / 8 + stats.intellect_bonus / 8) * factor

  local effect = target:create_effect(ability:name())
  effect:set_tag("disease")
  
  effect:add_attribute_bonus("Strength", -amount)
  effect:add_attribute_bonus("Dexterity", -amount)
  effect:add_attribute_bonus("Endurance", -amount)
  effect:add_attribute_bonus("Perception", -amount)
  
  local anim = target:create_particle_generator("heal")
  anim:set_moves_with_parent()
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0))
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  anim:set_gen_rate(anim:param(3.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(1.0, 1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  effect:add_anim(anim)
  
  effect:apply()
end
