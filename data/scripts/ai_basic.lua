function ai_action(parent, state)
    abilities = parent:abilities():can_activate():remove_kind("Special")
    abilities:sort_by_priority()
	
    hostiles = parent:targets():hostile():to_table()
    friendlies = parent:targets():friendly():to_table()

	items = parent:inventory():usable_items()
	
	failed_use_count = 0
	
    -- only loop at most 10 times
    for i = 1,10 do
		result = find_and_use_item(parent, items, hostiles, friendlies, failed_use_count)
		if result.done then
			_G.state = parent:state_wait(10)
			return
		end
	
        if abilities:is_empty() then
            break
        end

        result = find_and_use_ability(parent, abilities, hostiles, friendlies, failed_use_count)
        if result.done then
            _G.state = parent:state_wait(10)
            return
        end

		cur_len = abilities:len()
        abilities = abilities:can_activate()
		
		if cur_len == abilities:len() then
		  -- nothing was removed from the usable abilities, meaning nothing was activated
		  failed_use_count = failed_use_count + 1
		else
		  failed_use_count = 0
		end
    end

    if not parent:has_ap_to_attack() then
        _G.state = parent:state_end()
        return
    end

    target = find_target(parent, hostiles)
    if parent:can_reach(target) then
        parent:anim_weapon_attack(target, nil, true)
    else
        parent:move_towards_entity(target)
    end

    _G.state = parent:state_wait(10)
end

function find_and_use_item(parent, items, hostiles, friendlies, failed_use_count)
	for i = 1, #items do
		item = items[i]
		result = check_action(parent, item:ai_data(), hostiles, friendlies, failed_use_count)
		if result.done then
			return { done=true }
		end
			
		if not result.next_action then
			parent:use_item(item)
			return handle_targeter(parent, target)
		end
	end
	
	return { done=false }
end

function find_and_use_ability(parent, abilities, hostiles, friendlies, failed_use_count)
    abilities_table = abilities:to_table()
    for i = 1, #abilities_table do
        ability = abilities_table[i]

        result = check_action(parent, ability:ai_data(), hostiles, friendlies, failed_use_count)
        if result.done then
            return { done=true }
        end

        if not result.next_action then
			parent:use_ability(ability)
            return handle_targeter(parent, target)
        end
    end
	
	return { done=false }
end

-- just use the specified target if a targeter comes up
function handle_targeter(parent, target)
    if not game:has_targeter() then
        return { done=true }
    end

    x = target:x()
    y = target:y()

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
    target = get_best_target(parent, ai_data, hostiles, friendlies)

    if target == nil then
        return { done=false, next_action=true }
    end

    target_dist = 0
	
    if ai_data.range == "Personal" then
        return { done=false, next_action=false }
    elseif ai_data.range == "Reach" then
        if parent:can_reach(target) then
            return { done=false, next_action = false }
        end

        parent:move_towards_entity(target)
        return { done=true, next_action = false }
    elseif ai_data.range == "Short" then
        target_dist = 8 - failed_use_count * 2
    elseif ai_data.range == "Visible" then
        target_dist = parent:vis_dist() - 1 - failed_use_count * 3
		game:log("moving to within " .. target_dist .. " for long range")
    end
	
	dist = parent:dist_to_entity(target)
	
	if dist < target_dist then
	  return { done=false, next_action=false }
	else
	  parent:move_towards_entity(target, target_dist)
	  return { done=true, next_action=false }
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
    best_frac = 1.0
	best_target = nil
	
	for i = 1, #targets do
	    stats = targets[i]:stats()
		
		frac = stats.current_hp / stats.max_hp
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
    closest_dist = 1000
    closest_target = nil

    for i = 1, #targets do
        target = targets[i]
        dist = parent:dist_to_entity(target)
        if dist < closest_dist then
            closest_dist = dist
            closest_target = target
        end
    end

    return closest_target
end
