function on_activate(parent, ability)
  activate_no_check(parent, ability)
  
  ability:activate(parent)
  
  if check_spotted(parent, ability) then
    game:say_line("Failed to hide", parent)
	ability:deactivate(parent)
  end
end

function on_deactivate(parent, ability)
  ability:deactivate(parent)
end

function activate_no_ap(parent, ability)
  activate_no_check(parent, ability)
  ability:activate(parent, false)
end

function activate_no_check(parent, ability)
  local effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  effect:add_hidden()

  local cb = ability:create_callback(parent)
  cb:set_on_round_elapsed_fn("on_round_elapsed")
  cb:set_after_attack_fn("after_attack")
  cb:set_on_moved_fn("on_moved")
  effect:add_callback(cb)
  
  local anim = parent:create_color_anim()
  anim:set_color(anim:param(1.0),
                 anim:param(1.0),
                 anim:param(1.0),
                 anim:param(0.4))
  effect:add_color_anim(anim)
  effect:apply()
  
  game:play_sfx("sfx/rustle10")
end

function on_moved(parent, ability)
  if check_spotted(parent, ability) then
    game:say_line("Spotted!", parent)
	ability:deactivate(parent)
	game:cancel_blocking_anims()
	game:run_script_delayed("hide", "check_ai_activation", 0.1)
	game:play_sfx("sfx/rustle12")
  end
end

function on_round_elapsed(parent, ability)
  if check_spotted(parent, ability) then
    game:say_line("Spotted!", parent)
	ability:deactivate(parent)
	game:run_script_delayed("hide", "check_ai_activation", 0.1)
	game:play_sfx("sfx/rustle12")
  end
end

function check_ai_activation(parent)
  game:check_ai_activation(parent)
end

-- this function is used by external scripts that deactivate the hidden state
function deactivate(parent, ability)
  if not ability:is_active_mode(parent) then return end
  if parent:has_effect_with_tag("unspottable") then return end
  
  game:say_line("Spotted!", parent)
  ability:deactivate(parent)
  game:play_sfx("sfx/rustle12")
end

function after_attack(parent, ability)
  if parent:has_effect_with_tag("unspottable") then return end

  deactivate(parent, ability)
  game:run_script_delayed("hide", "check_ai_activation", 0.1)
end

function check_spotted(parent, ability)
  if parent:has_effect_with_tag("unspottable") then return end

  local stats = parent:stats()
  local parent_concealment = stats.concealment
  local parent_hide_level = 15 + parent:ability_level(ability) * 20 + stats.level * 2
  local break_even_distance = 5
  if parent:has_ability("hide_in_plain_sight") then
    break_even_distance = 0
  end
  
  local targets = parent:targets():hostile():visible()
  targets = targets:to_table()
  for i = 1, #targets do
    local target = targets[i]
	
	local target_perception = target:stats().perception * 5
	local dist = (target:dist_to_entity(parent) - break_even_distance) * 10
	
	-- game:log("hide check with " .. parent_hide_level .. " + " .. parent_concealment .. " + " .. dist .. " vs " .. target_perception)
	if parent_hide_level + parent_concealment + dist - target_perception < 0 then
	  return true
	end
  end
  
  return false
end
