function on_activate(parent, ability)
  stats = parent:stats()
  if not stats.attack_is_melee then
    return
  end

  targets = parent:targets():hostile():attackable()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets, selected_point)
  target = targets:first()
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:register_fn("after_attack")
  
  ability:activate(parent)
  parent:anim_special_attack(target, "Fortitude", 0, 0, "Raw", cb)
end

function after_attack(parent, ability, targets, hit)
  target = targets:first()
  
  if hit:is_graze() then
    target:change_overflow_ap(-20)
  elseif hit:is_hit() then
    target:change_overflow_ap(-40)
  elseif hit:is_crit() then
    target:change_overflow_ap(-60)
  end
  
  gen = target:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-0.75))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:activate()
end
