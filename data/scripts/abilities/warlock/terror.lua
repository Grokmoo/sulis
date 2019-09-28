function on_activate(parent, ability)
  local targets = parent:targets():hostile():reachable()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_reachable()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  ability:activate(parent)
  
  local hit = parent:special_attack(target, "Will", "Spell")
  local duration = ability:duration()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    duration = duration - 1
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = duration + 1
  end
  
  local effect = target:create_effect(ability:name(), duration)
  effect:set_tag("fear")
  effect:add_attack_disabled()
  effect:add_num_bonus("will", -20)
  
  local anim = target:create_color_anim()
  anim:set_color(anim:param(0.8),
                 anim:param(0.1),
                 anim:param(0.1),
                 anim:param(1.0))
  anim:set_color_sec(anim:param(0.3),
                     anim:param(0.0),
                     anim:param(0.0),
                     anim:param(0.0))
  effect:add_color_anim(anim)
  effect:apply()
end
