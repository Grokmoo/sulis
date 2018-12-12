function on_activate(parent, ability)
  targets = parent:targets():hostile():visible()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()

  duration = 0.8
  anim = parent:create_anim("crush", duration)
  anim:set_position(anim:param(target:center_x() - 0.5), anim:param(target:center_y() - 1.0))
  anim:set_particle_size_dist(anim:fixed_dist(2.0), anim:fixed_dist(2.0))
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

  stats = parent:stats()
  min_dmg = 15 + stats.caster_level / 2 + stats.intellect_bonus / 4
  max_dmg = 25 + stats.intellect_bonus / 2 + stats.caster_level
  
  parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 0, "Crushing")
end

