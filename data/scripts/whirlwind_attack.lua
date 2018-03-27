function on_activate(parent, ability)
  stats = parent:stats()
  if not stats.attack_is_melee then
    return
  end

  targeter = parent:create_targeter(ability)
  targeter:set_circle(stats.attack_distance)
  targeter:add(parent)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:register_fn("on_anim_update")

  duration = 1.0

  gen = parent:create_anim("whirlwind", duration)
  gen:set_position(gen:param(parent:x() - 2.0), gen:param(parent:y() - 2.0))
  gen:set_particle_size_dist(gen:fixed_dist(4.0), gen:fixed_dist(4.0))
  gen:set_alpha(gen:param(1.0, 0.0, 0.0, -6.0))

  targets = targets:collect()
  duration_per_target = duration / (#targets + 1)
  for i = 1, #targets do
    gen:add_callback(cb, duration_per_target * i)
  end
  
  gen:activate()
  ability:activate(parent)
end

function on_anim_update(parent, ability, targets, index)
  targets = targets:collect()
  
  if targets[index]:is_valid() then
    parent:weapon_attack(targets[index])
  end
end

