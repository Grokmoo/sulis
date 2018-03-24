function on_activate(parent, ability)
  stats = parent:stats()
  if not stats.attack_is_melee then
    return
  end

  targets = parent:targets():hostile():attackable()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:register_fn("before_attack")
  
  -- Remove an additional point of AP beyond the standard attack
  parent:remove_ap(10)
  -- ability:activate(parent)
  parent:weapon_attack(target, cb)
end

function before_attack(parent, ability, targets)
  target = targets:first()
  stats = parent:stats()

  effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("accuracy", 25)
  effect:add_damage(10, 15)
  effect:apply()
  
  gen = target:create_anim("slash", 1.0)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-0.75))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:set_color(gen:param(1.0), gen:param(0.1), gen:param(0.1))
  gen:activate()
end
