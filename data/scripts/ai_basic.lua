function ai_action(parent, state)
    abilities = parent:abilities():can_activate():remove_kind("Special")
    abilities:sort_by_priority()
	
    hostiles = parent:targets():hostile()
    friendlies = parent:targets():friendly()

	failed_use_count = 0
	
    -- only check at most 10 actions
    for i = 1,10 do
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

function find_and_use_ability(parent, abilities, hostiles, friendlies, failed_use_count)
    abilities_table = abilities:to_table()
    for i = 1, #abilities_table do
        ability = abilities_table[i]

        result = check_ability(parent, ability, hostiles, friendlies, failed_use_count)
        if result.done then
            return { done=true }
        end

        if not result.next_ability then
            return use_ability(parent, ability, target)
        end
    end
end

-- just use the specified target if a targeter comes up
function use_ability(parent, ability, target)
    parent:use_ability(ability)

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

function check_ability(parent, ability, hostiles, friendlies, failed_use_count)
    target = nil
    ai_data = ability:ai_data()
    if ai_data.range == "Personal" then
        target = parent
    elseif ai_data.kind == "Damage" or ai_data.kind == "Debuff" then
        target = find_target(parent, hostiles)
    else
        target = find_target(parent, friendlies)
    end

    if target == nil then
        return { done=false, next_ability=true }
    end

    target_dist = 0
	
    if ai_data.range == "Personal" then
        return { done=false, next_ability=false }
    elseif ai_data.range == "Reach" then
        if parent:can_reach(target) then
            return { done=false, next_ability = false }
        end

        parent:move_towards_entity(target)
        return { done=true, next_ability = false }
    elseif ai_data.range == "Short" then
        target_dist = 8 - failed_use_count * 2
    elseif ai_data.range == "Visible" then
        target_dist = parent:vis_dist() - 1 - failed_use_count * 3
		game:log("moving to within " .. target_dist .. " for long range")
    end
	
	dist = parent:dist_to_entity(target)
	
	if dist < target_dist then
	  return { done=false, next_ability=false }
	else
	  parent:move_towards_entity(target, target_dist)
	  return { done=true, next_ability=false }
	end
end

-- Just finds the closest target for now
function find_target(parent, targets)
    closest_dist = 1000
    closest_target = nil

    targets = targets:to_table()
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
