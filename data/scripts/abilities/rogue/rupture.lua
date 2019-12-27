function on_activate(parent, ability)
  local targets = parent:targets():hostile():attackable()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_attackable()
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

  if hit:is_miss() then return end

  local effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("rupture")
  
  local cb = ability:create_callback(parent)
  cb:set_on_moved_fn("on_moved")
  cb:add_target(target)
  effect:add_callback(cb)

  local anim = target:create_particle_generator("sparkle")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(0.0), anim:param(0.0))
  anim:set_particle_size_dist(anim:fixed_dist(0.5), anim:fixed_dist(0.5))
  anim:set_gen_rate(anim:param(6.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(1.0, 1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0))
  effect:add_anim(anim)
  
  effect:apply()
end

function on_moved(parent, ability, targets)
  local target = targets:first()
  target:take_damage(parent, 1, 1, "Raw")
end
