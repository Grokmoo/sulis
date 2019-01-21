function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible_within(7)
  
  local targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  local target = targets:first()
  
  if target:has_effect_with_tag("ward") then
    target:remove_effects_with_tag("ward")
  else
    target:remove_effects_with_tag("magic_defense")
  end
  
  local anim = target:create_color_anim(1.0)
  anim:set_color(anim:param(1.0),
                 anim:param(1.0),
                 anim:param(1.0),
                 anim:param(1.0))
  anim:set_color_sec(anim:param(1.0, -1.0),
                     anim:param(0.0),
                     anim:param(1.0, -1.0),
                     anim:param(0.0))
  anim:activate()
end
