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

  target:inventory():set_locked(true)
  
  effect = target:create_effect(ability:name(), duration)
  effect:set_tag("polymorph")
  effect:add_abilities_disabled()
  effect:add_attack_disabled()
  effect:add_attribute_bonus("Intellect", -8)
  effect:add_attribute_bonus("Strength", -6)
  effect:add_num_bonus("defense", -20)
  
  local gen = target:create_image_layer_anim()
  gen:add_image("Ears", "empty")
  gen:add_image("Hair", "empty")
  gen:add_image("Beard", "empty")
  gen:add_image("Head", "empty")
  gen:add_image("Hands", "empty")
  gen:add_image("HeldMain", "empty")
  gen:add_image("HeldOff", "empty")
  gen:add_image("Background", "creatures/chicken0")
  gen:add_image("Torso", "empty")
  gen:add_image("Legs", "empty")
  gen:add_image("Feet", "empty")
  gen:add_image("Foreground", "empty")
  effect:add_image_layer_anim(gen)

  effect:apply()
  
  local anim = target:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()
end

function on_removed(parent, ability)
  local inv = parent:inventory()
  inv:set_locked(false)
   
  local anim = parent:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()
end