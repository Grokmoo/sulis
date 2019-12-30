function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible():attackable()
  
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
  cb:set_before_attack_fn("create_parent_bonus")
  
  ability:activate(parent)
  parent:anim_weapon_attack(target, cb)
end

function create_parent_bonus(parent, ability, targets)
  local effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("crit_chance", 100)
  effect:apply()
end