function on_activate(parent, ability)
  local targets = parent:targets():hostile():attackable()
  
  local targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_after_attack_fn("create_target_effect")
  
  ability:activate(parent)
  parent:anim_weapon_attack(target, cb)
end

function create_target_effect(parent, ability, targets, hit)
  local target = targets:first()

  if hit:is_miss() then return end
  
  local duration = ability:duration()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    duration = duration / 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = duration * 3 / 2
  end
  
  local effect = target:create_effect(ability:name(), duration)
  effect:set_tag("poison")
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_round_elapsed_fn("apply_damage")
  effect:add_callback(cb)
  
  local anim = target:create_color_anim()
  anim:set_color(anim:param(0.0),
                 anim:param(1.0),
                 anim:param(0.0),
                 anim:param(1.0))
  effect:add_color_anim(anim)
  effect:apply()
end

function apply_damage(parent, ability, targets)
  local stats = parent:stats()
  local target = targets:first()
  
  local max_dmg = 6 + stats.level / 2
  target:take_damage(parent, 3, max_dmg, "Raw")
end