function on_activate(parent, ability)
  targets = parent:targets():hostile():visible()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()

  duration = 0.6
  anim = parent:create_anim("shock", duration)
  anim:set_position(anim:param(target:center_x() - 1.5), anim:param(target:center_y() - 1.5))
  anim:set_particle_size_dist(anim:fixed_dist(3.0), anim:fixed_dist(3.0))
  anim:set_alpha(anim:param(1.0))
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_anim_update_fn("attack_target")
  anim:add_callback(cb, duration - 0.2)
  
  anim:activate()
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  target = targets:first()

  parent:special_attack(target, "Reflex", "Spell", 20, 30, 0, "Electrical")
end

