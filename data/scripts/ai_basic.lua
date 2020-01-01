MOVE_THRESHOLD=0.1

function ai_action(parent)
    game:log("AI turn for " .. parent:name())
    game:log("  Current AP " .. tostring(parent:stats().current_ap))

    local abilities = parent:abilities():can_activate():remove_kind("Special")
    abilities:sort_by_priority()

    local hostiles = parent:targets():hostile()
    local friendlies = parent:targets():friendly():to_table()

    game:log("  Got " .. tostring(abilities:len()) .. " abilities")
    game:log("  Got " .. tostring(hostiles:num_targets()) .. " hostiles")
    game:log("  Got " .. tostring(#friendlies) .. " friendlies")

    if parent:has_effect_with_tag("fear") then
        game:log("  Running away due to fear")
        attempt_run_away(parent, hostiles:visible():to_table())
        return parent:state_end()
    end

    if check_swap_weapons(parent, hostiles).done then
        return parent:state_wait(10)
    end

    local items = parent:inventory():usable_items()

    game:log("  Got " .. tostring(#items) .. " items")

    hostiles = hostiles:to_table()

    local failed_use_count = 0

    -- only loop at most 10 times
    for i = 1,10 do
        local result = find_and_use_item(parent, items, hostiles, friendlies, failed_use_count)
        if result.done then
            game:log("  Item used or moved")
            return parent:state_wait(10)
        end

        if abilities:is_empty() then
            break
        end

        local result = find_and_use_ability(parent, abilities, hostiles, friendlies, failed_use_count)
        if result.done then
            game:log("  Ability used or moved")
            return parent:state_wait(10)
        end

        local cur_len = abilities:len()
        abilities = abilities:can_activate()

        if cur_len == abilities:len() then
            game:log("  Unable to use ability.  Increment failed_use_count")
            -- nothing was removed from the usable abilities, meaning nothing was activated
            failed_use_count = failed_use_count + 1
        else
            failed_use_count = 0
        end
    end
    game:log("  Unable to use any abilities.  Attempting attack.")

    if not parent:has_ap_to_attack() then
        game:log("  No AP remaining.  End")
        return parent:state_end()
    end

    local target = check_for_valid_target(parent, hostiles)
    if target == nil then
        game:log("  No valid attack target.  End")
        return parent:state_end()
    end

    local result = check_move_for_attack(parent, target)
    if result.attack then
        game:log("  Attack target " .. target:name())
        parent:anim_weapon_attack(target, nil, true)
    end

    if result.done then
        game:log("  Done")
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
        game:log("    Melee attack")
        if parent:is_within_attack_dist(target) then
            game:log("    In range")
            return { attack=true }
        end

        game:log("    Attempt move towards target")
        if not parent:move_towards_entity(target) then
            game:log("    Unable to move.")
            return { attack=false, done=true }
        end
    else
        game:log("    Ranged attack")
        local dist = parent:dist_to_entity(target)
        local target_dist = parent:stats().attack_distance - 1

        game:log("At dist " .. tostring(dist) .. " target dist is " .. tostring(target_dist))
        if dist > target_dist then
            game:log("    Attempt move towards target")
            if not parent:move_towards_entity(target, target_dist) then
                game:log("    Unable to move.")
                return { attack=false, done=true }
            end

            return { attack=false, done=false }
        end

        if not parent:has_visibility(target) then
            game:log("    No visibility.  Move towards")
            if not parent:move_towards_entity(target, dist - 2) then
                game:log("    Unable to move.")
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
        game:log("  Attempting swap weapons")
        if parent:swap_weapons() then
            return { done=true }
        end
    end

    return { done=false }
end

function find_and_use_item(parent, items, hostiles, friendlies, failed_use_count)
    for i = 1, #items do
        local item = items[i]

        game:log("    Checking item " .. item:name())
        local result = check_action(parent, item:ai_data(), hostiles, friendlies, failed_use_count)
        if result.done then
            return { done=true }
        end

        if result.target then
            parent:use_item(item)
            return handle_targeter(parent, result.target, item:ai_data(), hostiles, friendlies)
        end
    end

    return { done=false }
end

function find_and_use_ability(parent, abilities, hostiles, friendlies, failed_use_count)
    local abilities_table = abilities:to_table()
    for i = 1, #abilities_table do
        local ability = abilities_table[i]

        game:log("    Checking ability " .. ability:name())
        local result = check_action(parent, ability:ai_data(), hostiles, friendlies, failed_use_count)
        if result.done then
            return { done=true }
        end

        if result.target then
            game:log("      Use ability")
            parent:use_ability(ability)
            return handle_targeter(parent, result.target, ability:ai_data(), hostiles, friendlies)
        end
    end

    return { done=false }
end

-- find the best target for the given targeter
function handle_targeter(parent, closest_target, ai_data, hostiles, friendlies)
    if not game:has_targeter() then
        return { done=false }
    end

    game:log("      Attempting to handle targeter")

    if ai_data.kind == "Heal" then
        return activate_targeter(closest_target)
    end

    -- want hostiles and don't want friendlies for damage or debuff, the opposite for others
    local relationship_modifier = 1
    if ai_data.kind == "Damage" or ai_data.kind == "Debuff" then
        relationship_modifier = -1
    end

    -- get set of possible targeter targets
    local to_check = nil
    if game:is_targeter_free_select() then
        if relationship_modifier == -1 then
            to_check = hostiles
        else
            to_check = friendlies
        end
    else
        to_check = game:get_targeter_selectable():to_table()
    end

    if to_check == nil then
        game:log("      No available targets")
        return { done=false }
    end

    game:log("      Got " .. tostring(#to_check) .. " targets ")
    game:log("      and relationship modifier " .. tostring(relationship_modifier))

    -- find the best scored target
    local best_score = 0
    local best_target = nil
    for i = 1,#to_check do
        local cur_score = 0
        local try_target = to_check[i]
        if game:check_targeter_position(try_target:x(), try_target:y()) then
            local potential_targets = game:get_targeter_affected():to_table()
            for j = 1,#potential_targets do
                local target = potential_targets[j]
                cur_score = cur_score + parent:get_relationship(target) * relationship_modifier
            end
        end

        if cur_score > best_score then
            best_score = cur_score
            best_target = try_target
        end
    end

    game:log("      Best score was " .. tostring(best_score))

    -- fire targeter on best scored target.  if none with score greater than 0 was found,
    -- will just return
    return activate_targeter(best_target)
end

function activate_targeter(target)
    if target == nil then
        game:log("      No target.")
        game:cancel_targeter()
        return { done=false }
    end

    game:log("      Found best target " .. tostring(target:name()))

    local x = target:x()
    local y = target:y()

    if game:check_targeter_position(x, y) then
        game:log("      Activate targeter")
        game:activate_targeter()
        return { done=true }
    else
        game:log("      Cancel targeter")
        game:cancel_targeter()
        return { done=false }
    end
end

function check_action(parent, ai_data, hostiles, friendlies, failed_use_count)
    local target = get_close_target(parent, ai_data, hostiles, friendlies)

    if target == nil then
        game:log("      No valid target")
        return { done=false, target=nil }
    end

    game:log("      Got target " .. target:name())

    local target_dist = 0

    if ai_data.range == "Personal" then
        game:log("      Personal use ability")
        return { done=false, target=parent }
    elseif ai_data.range == "Touch" then
        if parent:is_within_touch_dist(target) then
            game:log("      Touch range ability within range")
            return { done=false, target=target }
        end

        local stats = parent:stats()
        game:log("      Moving to within touch range")
        parent:move_towards_entity(target, stats.touch_distance - MOVE_THRESHOLD)
        return { done=true }
    elseif ai_data.range == "Attack" then
        if parent:is_within_attack_dist(target) then
            game:log("      Attack range ability within range")
            return { done=false, target=target }
        end

        game:log("      Moving to within attack range")
        parent:move_towards_entity(target)
        return { done=true }
    elseif ai_data.range == "Short" then
        target_dist = 8 - failed_use_count * 2
    elseif ai_data.range == "Visible" then
        target_dist = parent:vis_dist() - 1 - failed_use_count * 3
    end

    local dist = parent:dist_to_entity(target)

    game:log("      Ability target dist of " .. tostring(target_dist))
    game:log("      current dist " .. tostring(dist))
    if dist <= target_dist then
        game:log("      In Range")
        return { done=false, target=target}
    else
        game:log("      Moving to within range")
        parent:move_towards_entity(target, target_dist - MOVE_THRESHOLD)
        return { done=true }
    end
end

HEALING_FRAC = 0.5

function get_close_target(parent, ai_data, hostiles, friendlies)
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
        return check_for_valid_target(parent, hostiles)
    elseif ai_data.kind == "Heal" then
        return find_heal_target(parent, friendlies)
    else
        return check_for_valid_target(parent, friendlies)
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
function check_for_valid_target(parent, targets)
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
    -- local target = targets:first()
	-- game:log(parent:name() .. " damaged by " .. target:name() .. ": "
	--     .. hit:kind() .. " for " .. hit:total_damage() .. " damage.")
end
