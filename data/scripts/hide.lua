function on_activate(parent, ability)
  effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  effect:add_hidden()

  cb = ability:create_callback(parent)
  cb:set_on_round_elapsed_fn("on_round_elapsed")
  cb:set_after_attack_fn("after_attack")
  effect:add_callback(cb)
  
  anim = parent:create_color_anim()
  anim:set_color(anim:param(1.0),
                 anim:param(1.0),
                 anim:param(1.0),
                 anim:param(0.4))
  effect:apply_with_color_anim(anim)

  ability:activate(parent)
  
  if check_spotted(parent, ability) then
    game:say_line("Failed to hide", parent)
	ability:deactivate(parent)
  end
end

function on_round_elapsed(parent, ability)
  if check_spotted(parent, ability) then
    game:say_line("Spotted!", parent)
	ability:deactivate(parent)
  end
end

function after_attack(parent, ability)
  game:say_line("Spotted!", parent)
  ability:deactivate(parent)
end

function check_spotted(parent, ability)
  parent_concealment = parent:stats().concealment
  parent_hide_level = parent:ability_level(ability) * 20
  break_even_distance = 8
  if parent:get_ability("hide_in_plain_sight") ~= nil then
    break_even_distance = 0
  end
  
  targets = parent:targets():hostile():visible()
  targets = targets:to_table()
  for i = 1, #targets do
    target = targets[i]
	
	target_perception = target:stats().perception * 5
	dist = (target:dist_to_entity(parent) - break_even_distance) * 10
	
	-- game:log("hide check with " .. parent_hide_level .. " + " .. parent_concealment .. " + " .. dist .. " vs " .. target_perception)
	if parent_hide_level + parent_concealment + dist - target_perception < 0 then
	  return true
	end
  end
  
  return false
end