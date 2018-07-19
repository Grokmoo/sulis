function on_activate(parent, ability)
  stats = parent:stats()
  if not stats.attack_is_melee then
    game:say_line("You must have a melee weapon equipped.", parent)
    return
  end

  targets = parent:targets():hostile():attackable()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()
  
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_after_attack_fn("attack2")
  
  ability:activate(parent)
  parent:anim_weapon_attack(target, cb)
end

function attack2(parent, ability, targets)
  target = targets:first()
  if not target:is_valid() then return end
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_before_attack_fn("apply_penalty1")
  cb:set_after_attack_fn("attack3")
  parent:anim_weapon_attack(target, cb)
end

function attack3(parent, ability, targets)
  target = targets:first()
  if not target:is_valid() then return end
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_before_attack_fn("apply_penalty2")
  cb:set_after_attack_fn("attack4")
  parent:anim_weapon_attack(target, cb)
end

function attack4(parent, ability, targets)
  target = targets:first()
  if not target:is_valid() then return end
  cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_before_attack_fn("apply_penalty3")
  parent:anim_weapon_attack(target, cb)
end

function apply_penalty1(parent, ability, targets)
  effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("accuracy", -20)
  effect:apply()
end

function apply_penalty2(parent, ability, targets)
  effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("accuracy", -40)
  effect:apply()
end

function apply_penalty3(parent, ability, targets)
  effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("accuracy", -60)
  effect:apply()
end
