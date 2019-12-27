function ai_action(parent)
    local abilities = parent:abilities():can_activate():remove_kind("Special")
    abilities:sort_by_priority()
	
    local hostiles = parent:targets():hostile()
    local friendlies = parent:targets():friendly():to_table()

    if parent:has_effect_with_tag("fear") then
	    attempt_run_away(parent, hostiles:visible():to_table())
	    return parent:state_end()
	end

	if check_swap_weapons(parent, hostiles).done then
		return parent:state_wait(10)
	end
	
	local items = parent:inventory():usable_items()
	
	hostiles = hostiles:to_table()
	
	local failed_use_count = 0
	
    -- only loop at most 10 times
    for i = 1,10 do
		local result = find_and_use_item(parent, items, hostiles, friendlies, failed_use_count)
		if result.done then
			return parent:state_wait(10)
		end
	
        if abilities:is_empty() then
            break
        end

        local result = find_and_use_ability(parent, abilities, hostiles, friendlies, failed_use_count)
        if result.done then
            return parent:state_wait(10)
        end

		local cur_len = abilities:len()
        abilities = abilities:can_activate()
		
		if cur_len == abilities:len() then
		  -- nothing was removed from the usable abilities, meaning nothing was activated
		  failed_use_count = failed_use_count + 1
		else
		  failed_use_count = 0
		end
    end

    if not parent:has_ap_to_attack() then
        return parent:state_end()
    end

    local target = find_target(parent, hostiles)
	if target == nil then
	  return parent:state_end()
	end
	
	local result = check_move_for_attack(parent, target)
	if result.attack then
		parent:anim_weapon_attack(target, nil, true)
	end
	
	if result.done then
		return parent:state_end()
	else
		return parent:state_wait(10)
	end
end

function attempt_run_away(parent, hostiles)
	local parent_x = parent:x()
	local parent_y = parent:y()

    -- calculate mean of the angles using the atan2 of sin and cos formula
	local total_x_component = 0.0
	local total_y_component = 0.0
	
	for i = 1, #hostiles do
	    local target = hostiles[i]
		local angle = game:atan2(parent_x - target:x(), parent_y - target:y())
	    total_y_component = total_y_component + math.sin(angle)
		total_x_component = total_x_component + math.cos(angle)
	end

    local average_angle = game:atan2(total_x_component / #hostiles, total_y_component / #hostiles)
	
	local angle_cos = math.cos(average_angle + math.pi)
	local angle_sin = math.sin(average_angle + math.pi)
	
	-- try to move in the direction some distance away
	-- if unable, try moving in the general direction with less and less accuracy
	local x_diff = math.floor(angle_cos * 12 + 0.5)
	local y_diff = math.floor(angle_sin * 12 + 0.5)
	  
	local x = parent:x() - x_diff
	local y = parent:y() - y_diff
	
	for thresh = 1, 10 do
	  if parent:move_towards_point(x, y, thresh) then
	    return { done=true }
	  end
	end
end

function check_move_for_attack(parent, target)
    if not parent:stats().attack_is_ranged then
	    if parent:is_within_attack_dist(target) then
		    return { attack=true }
		end
		
		if not parent:move_towards_entity(target) then
			return { attack=false, done=true }
		end
	else
		local dist = parent:dist_to_entity(target)
		local target_dist = parent:vis_dist() - 2
		
		if dist > target_dist then
			if not parent:move_towards_entity(target, target_dist) then
				return { attack=false, done=true }
			end
			
			return { attack=false, done=false }
		end
		
		if not parent:has_visibility(target) then
			if not parent:move_towards_entity(target, dist - 2) then
				return { attack=false, done=true }
			end
			
			return { attack=false, done=false }
		end
		
		return { attack=true }
	end
	
	return { attack=false, done=false }
end

function check_swap_weapons(parent, hostiles)
	if parent:inventory():weapon_style() ~= "Ranged" then
		return { done=false }
	end
	
	if hostiles:threatening():is_empty() then
		return { done=false }
	end
	
	if not parent:inventory():has_alt_weapons() then
		return { done=false }
	end
	
	if parent:inventory():alt_weapon_style() ~= "Ranged" then
		if parent:swap_weapons() then
			return { done=true }
		end
	end
	
	return { done=false }
end

function find_and_use_item(parent, items, hostiles, friendlies, failed_use_count)
	for i = 1, #items do
		local item = items[i]
		local result = check_action(parent, item:ai_data(), hostiles, friendlies, failed_use_count)
		if result.done then
			return { done=true }
		end
			
		if result.target then
			parent:use_item(item)
			return handle_targeter(parent, result.target)
		end
	end
	
	return { done=false }
end

function find_and_use_ability(parent, abilities, hostiles, friendlies, failed_use_count)
    local abilities_table = abilities:to_table()
    for i = 1, #abilities_table do
        local ability = abilities_table[i]

        local result = check_action(parent, ability:ai_data(), hostiles, friendlies, failed_use_count)
        if result.done then
            return { done=true }
        end

        if result.target then
			parent:use_ability(ability)
            return handle_targeter(parent, result.target)
        end
    end
	
	return { done=false }
end

-- just use the specified target if a targeter comes up
function handle_targeter(parent, target)
    if not game:has_targeter() then
        return { done=true }
    end

    local x = target:x()
    local y = target:y()

    if game:check_targeter_position(x, y) then
        game:activate_targeter()
		return { done=true }
    else
        -- TODO need to search for another target here
        game:cancel_targeter()
		return { done=false }
    end
end

function check_action(parent, ai_data, hostiles, friendlies, failed_use_count)
    local target = get_best_target(parent, ai_data, hostiles, friendlies)

    if target == nil then
        return { done=false, target=nil }
    end

    local target_dist = 0
	
    if ai_data.range == "Personal" then
        return { done=false, target=parent }
    elseif ai_data.range == "Reach" then
        if parent:is_within_touch_dist(target) then
            return { done=false, target=target }
        end

        parent:move_towards_entity(target)
        return { done=true }
    elseif ai_data.range == "Short" then
        target_dist = 8 - failed_use_count * 2
    elseif ai_data.range == "Visible" then
        target_dist = parent:vis_dist() - 1 - failed_use_count * 3
    end
	
	local dist = parent:dist_to_entity(target)
	
	if dist < target_dist then
	  return { done=false, target=target}
	else
	  parent:move_towards_entity(target, target_dist)
	  return { done=true }
	end
end

HEALING_FRAC = 0.5

function get_best_target(parent, ai_data, hostiles, friendlies)
    if ai_data.range == "Personal" then
		if ai_data.kind == "Heal" then
			stats = parent:stats()
			if stats.current_hp / stats.max_hp < HEALING_FRAC then
				return parent
			else
				return nil
			end
		else
			return parent
		end
    elseif ai_data.kind == "Damage" or ai_data.kind == "Debuff" then
        return find_target(parent, hostiles)
	elseif ai_data.kind == "Heal" then
	    return find_heal_target(parent, friendlies)
    else
        return find_target(parent, friendlies)
    end
end

function find_heal_target(parent, targets)
    local best_frac = 1.0
	local best_target = nil
	
	for i = 1, #targets do
	    local stats = targets[i]:stats()
		
		local frac = stats.current_hp / stats.max_hp
		if frac < best_frac then
		    best_frac = frac
			best_target = targets[i]
		end
	end
	
	if best_frac < HEALING_FRAC then
		return best_target
	else
		return nil
	end
end

-- Just finds the closest target for now
function find_target(parent, targets)
    local closest_dist = 1000
    local closest_target = nil

    for i = 1, #targets do
        local target = targets[i]
        local dist = parent:dist_to_entity(target)
        if dist < closest_dist then
            closest_dist = dist
            closest_target = target
        end
    end

    return closest_target
end

-- OnDamaged script hook
function on_damaged(parent, targets, hit)
    local target = targets:first()
	
	-- game:log(parent:name() .. " damaged by " .. target:name() .. ": "
	--     .. hit:kind() .. " for " .. hit:total_damage() .. " damage.")
end
