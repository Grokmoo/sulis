function on_activate(parent, ability)
  targets = parent:targets():hostile():attackable()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()
  
  cb = ability:create_callback(parent, "on_attack")
  cb:add_target(target)
  
  -- Remove an additional point of AP beyond the standard attack
  parent:remove_ap(10)
  -- ability:activate(parent)
  parent:attack(target, cb)
end

function on_attack(parent, ability, targets)
  stats = parent:stats()

  effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("accuracy", 25)
  effect:add_damage(10, 15)
  effect:apply()
end
