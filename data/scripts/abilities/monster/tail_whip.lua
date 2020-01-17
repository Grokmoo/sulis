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
  cb:set_after_attack_fn("create_stun_effect")
  
  ability:activate(parent)
  parent:anim_weapon_attack(target, cb)
end

function create_stun_effect(parent, ability, targets, hit)
  if hit:is_miss() then return end

  local target = targets:first()
  
  -- compute the max target pushback distance
  local pushback_dist = 4 + 2 * parent:width() - 2 * target:width()
  if parent:ability_level(ability) > 1 then
    pushback_dist = pushback_dist + 3
  end
  
  local point = {x = parent:x(), y = parent:y()}
  local direction = 1
  
  push_target(pushback_dist, target, hit, point, direction)
end

--INCLUDE push_target
