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
  cb:set_before_attack_fn("create_parent_effect")
  
  ability:activate(parent)
  parent:anim_weapon_attack(target, cb)
end

function create_parent_effect(parent, ability, targets)
  local target = targets:first()
  local stats = parent:stats()

  local effect = parent:create_effect(ability:name(), 0)
  
  effect:add_num_bonus("melee_accuracy", 20 + stats.level)
  effect:add_damage(10, 15 + stats.level / 2)
  effect:apply()
  
  local gen = target:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-0.75))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:activate()
end
