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
  targets = targets:collect()
  for i = 1, #targets do
    parent:weapon_attack(targets[i])
  end
  
  ability:activate(parent)
end
