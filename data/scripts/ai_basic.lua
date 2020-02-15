MIN_MULTIPLE_SCORE= 2
MOVE_THRESHOLD = 0.1
HEALING_FRAC = 0.5
WAIT_TIME = 10
MAX_MOVE_LEN = 60

-- This AI reads the following params
-- AttackWhenHasAbilitiesChance value from 0 to 100.  Percent chance to use a standard attack
-- when the parent has a potential ability use
-- AlwaysUseAbilityPriority integer value.  Abilities with this ai_priority or less will always
-- be used over standard attacks, disregarding AttackWhenHasAbilitiesChance
-- MeleeAttackMoveTries integer from 0 to 5.  When greater than 0, the parent will attempt to
-- move closer to targets even if they cannot directly attack, up to the specified distance
-- multiplied by the parent size.  This normally will make it easy
-- for the player to dispatch them with area of effect attacks.

function ai_action(parent, params)
    -- set default value of 0 for all params
    local meta_default = { __index = function() return 0 end }
	setmetatable(params, meta_default)

    game:log("AI turn for " .. parent:id())
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
        return end_turn(parent)
    end

    if check_swap_weapons_to_melee(parent, hostiles).done then
        return parent:state_wait(WAIT_TIME)
    end

    local items = parent:inventory():usable_items()

    game:log("  Got " .. tostring(#items) .. " items")

    hostiles = hostiles:to_table()

    local failed_use_count = 0

    if not parent:has_flag("ai_force_attack") then
      local skip_abilities_chance = math.random(100)
	  if math.random(0, 99) < params["AttackWhenHasAbilitiesChance"] then
	    parent:set_flag("ai_force_attack", "true")
      else
	    parent:set_flag("ai_force_attack", "false")
      end
	end

    -- only check items and abilities at most 10 times
    for i = 1,10 do
        local result = find_and_use_item(parent, items, hostiles, friendlies, failed_use_count)
        if result.done then
            game:log("  Item used or moved")
            return parent:state_wait(WAIT_TIME)
        end

        if abilities:is_empty() then
            break
        end

        local result = find_and_use_ability(parent, params, abilities, hostiles,
            friendlies, failed_use_count)
        if result.done then
            game:log("  Ability used or moved")
            return parent:state_wait(WAIT_TIME)
        end

        if result.no_abilities then
            game:log("  No abilities currently available.")
            break
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
        return end_turn(parent)
    end

    local targets = sort_attack_targets(parent, hostiles)
    if targets == nil then
        game:log("  No valid attack target.  End")
        return end_turn(parent)
    end

    local max_retries = params["MeleeAttackMoveTries"]

    for retry = 0, max_retries do
        game:log("Trying to find attack target attempt " .. tostring(retry))
        for i = 1, #targets do
            local target = targets[i]
            game:log("  Checking for attack against " .. target:id())

            local result = check_move_for_attack(parent, target, retry)
            if result.attack then
                game:log("  Perform attack")
                parent:anim_weapon_attack(target, nil, true)
                parent:clear_flag("ai_force_attack")

                return parent:state_wait(WAIT_TIME)
            end

            if result.moved then
                game:log("  Moved.")
                return parent:state_wait(WAIT_TIME)
            end
        end
    end

    -- if using a melee weapon and unable to attack any targets
    if check_swap_weapons_to_ranged(parent).swapped then
        game:log("    Swapping to ranged attack.")
        return parent:state_wait(WAIT_TIME)
    else
        game:log("    Unable to swap weapons.  Nothing left to try")
    end

    return end_turn(parent)
end

function end_turn(parent)
    return parent:state_end()
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

function check_move_for_attack(parent, target, attempt)
    if not parent:stats().attack_is_ranged then
        game:log("    Melee attack")
        if parent:is_within_attack_dist(target) then
            game:log("    In range")
            return { attack=true }
        end

        local increase = attempt * math.max(parent:width(), parent:height())

        game:log("    Attempt move towards target")
        local target_dist = parent:stats().attack_distance
        if not parent:move_towards_entity(target, target_dist + increase, MAX_MOVE_LEN) then
            game:log("    Unable to move.")

            return { attack=false }
        end

        return { attack=false, moved=true }
    else
        if attempt > 0 then
          -- Don't retry ranged attacks since we aren't doing anything different
          game:log("  Not retrying ranged attacks.")
          return { attack=false }
        end

        game:log("    Ranged attack")
        local dist = parent:dist_to_entity(target)
        local target_dist = parent:stats().attack_distance - 1

        game:log("At dist " .. tostring(dist) .. " target dist is " .. tostring(target_dist))
        if dist > target_dist then
            game:log("    Attempt move towards target")
            if not parent:move_towards_entity(target, target_dist, MAX_MOVE_LEN) then
                game:log("    Unable to move.")
                return { attack=false }
            end

            return { attack=false, moved=true }
        end

        if not parent:has_visibility(target) then
            game:log("    No visibility.  Move towards")
            if not parent:move_towards_entity(target, dist - 2, MAX_MOVE_LEN) then
                game:log("    Unable to move.")
                return { attack=false }
            end

            return { attack=false, moved=true }
        end

        return { attack=true }
    end
end

function check_swap_weapons_to_ranged(parent)
    if parent:stats().attack_is_ranged then
        return { swapped=false }
    end

    if not parent:inventory():has_alt_weapons() then
        return { swapped=false }
    end

    if parent:inventory():alt_weapon_style() == "Ranged" then
        if parent:swap_weapons() then
            return { swapped=true }
        end
    end

    return { swapped=false }
end

function check_swap_weapons_to_melee(parent, hostiles)
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
            game:log("      Use item")
            parent:use_item(item)
            local result = handle_targeter(parent, result.target, item:ai_data(), hostiles, friendlies)

            if result.done then
                return { done=true }
            end
        end
    end

    return { done=false }
end

function find_and_use_ability(parent, params, abilities, hostiles, friendlies, failed_use_count)
    local abilities_table = abilities:to_table()
    for i = 1, #abilities_table do
        local ability = abilities_table[i]
        local ai_data = ability:ai_data()

        if parent:get_flag("ai_force_attack") == "true" then
		    -- don't skip max priority ability
			if ai_data.priority > params["AlwaysUseAbilityPriority"] then 
				game:log("  Skipping abilities")
				return { done=false, no_abilities=true }
			end
		end

        game:log("    Checking ability " .. ability:name())
        local result = check_action(parent, ai_data, hostiles, friendlies, failed_use_count)
        if result.done then
            return { done=true }
        end

        if result.target then
            game:log("      Use ability")
            parent:use_ability(ability)
            local result = handle_targeter(parent, result.target, ai_data, hostiles, friendlies)

            if result.done then
                return { done=true }
            end
        end
    end

    return { done=false, no_abilities=true }
end

-- find the best target for the given targeter
function handle_targeter(parent, closest_target, ai_data, hostiles, friendlies)
    if not game:has_targeter() then
        return { done=false, do_next=true }
    end

    game:log("      Attempting to handle targeter")

    if ai_data.kind == "Heal" then
        return activate_targeter({ x=closest_target:x(), y=closest_target:y() })
    elseif ai_data.kind == "Summon" then
        game:log("      Attempting to activate summon")
        return activate_targeter(find_targeter_position_nearby(closest_target))
    end

    -- want hostiles and don't want friendlies for damage or debuff, the opposite for others
    local relationship_modifier = 1
    if ai_data.kind == "Damage" or ai_data.kind == "Debuff" or ai_data.kind == "Summon" then
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
        game:cancel_targeter()
        return { done=false, do_next=true }
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
                local weight = compute_weight(parent, target)
                cur_score = cur_score + weight * relationship_modifier
            end
        end

        if cur_score > best_score then
            best_score = cur_score
            best_target = try_target
        end
    end

    game:log("      Best score was " .. tostring(best_score))

    if ai_data.group == "Multiple" then
        if best_score < MIN_MULTIPLE_SCORE then
            game:log("      Minimum score for multi-target ability not reached.  cancel.")
            game:cancel_targeter()
            return { done=false, do_next=true }
        end
    end

    if best_target == nil then
        game:log("      No target.")
        game:cancel_targeter()
        return { done=false, do_next=true }
    end

    game:log("      Found best target " .. best_target:id())

    -- fire targeter on best scored target.  if none with score greater than 0 was found,
    -- will just return
    return activate_targeter({ x=best_target:x(), y=best_target:y() })
end

function find_targeter_position_nearby(target)
    game:log("      Looking for valid target near " .. tostring(target:id()))

    local base_x = target:x() - 5
    local base_y = target:y() - 5

    for y = 1, 9, 2 do
        for x = 1, 9, 2 do
            local pos_x = x + base_x
            local pos_y = y + base_y
            if game:check_targeter_position(pos_x, pos_y) then
                game:log("      Found targeter pos at " .. tostring(pos_x) .. ", " .. tostring(pos_y))
                return { x=pos_x, y=pos_y }
            end
        end
    end

    game:log("      Unable to locate valid targeter pos")
    return nil
end

function activate_targeter(target)
    if target == nil then
        game:log("      No target.")
        game:cancel_targeter()
        return { done=false, do_next=true }
    end

    if game:check_targeter_position(target.x, target.y) then
        game:log("      Activate targeter")
        game:activate_targeter()
        return { done=true }
    else
        game:log("      Cancel targeter")
        game:cancel_targeter()
        return { done=false, do_next=true }
    end
end

function check_action(parent, ai_data, hostiles, friendlies, failed_use_count)
    local target = get_usable_target(parent, ai_data, hostiles, friendlies)

    if target == nil then
        game:log("      No valid target")
        return { done=false, target=nil }
    end

    game:log("      Got target " .. target:id())

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
        return check_move_towards(parent, target, stats.touch_distance - MOVE_THRESHOLD)
    elseif ai_data.range == "Attack" then
        if parent:is_within_attack_dist(target) then
            game:log("      Attack range ability within range")
            return { done=false, target=target }
        end

        local stats = parent:stats()
        game:log("      Moving to within attack range")
        return check_move_towards(parent, target, stats.attack_distance - MOVE_THRESHOLD)
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
        return check_move_towards(parent, target, target_dist - MOVE_THRESHOLD)
    end
end

function check_move_towards(parent, target, dist)
    if parent:move_towards_entity(target, dist, MAX_MOVE_LEN) then
        return { done=true }
    else
        game:log("      Unable to path towards " .. target:id())
        return { done=false }
    end
end

function get_usable_target(parent, ai_data, hostiles, friendlies)
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
    elseif ai_data.kind == "Damage" or ai_data.kind == "Debuff" or ai_data.kind == "Summon" then
        return find_closest_target(parent, hostiles)
    elseif ai_data.kind == "Heal" then
        return find_heal_target(parent, friendlies)
    else
        return find_closest_target(parent, friendlies)
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
function find_closest_target(parent, targets)
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

function sort_attack_targets(parent, hostiles)
    if hostiles == nil then return nil end

    local ranked = {}
    local scores = {}

    game:log("  Sorting " .. tostring(#hostiles) .. " potential attack targets")
    for i = 1, #hostiles do
        local target = hostiles[i]
        local score = compute_weight(parent, target)
        game:debug("        Got score of " .. tostring(score) .. " for " .. target:id())
        ranked[score] = target
        table.insert(scores, score)
    end

    table.sort(scores)

    local out = {}
    for i = 1, #scores do
        local score = scores[i]
        table.insert(out, ranked[score])
    end

    return out
end

function compute_weight(parent, target)
    local base = parent:get_relationship(target)
    local target_stats = target:stats()

    local modifiers = 0

    -- threatening hostiles are higher priority
    if parent:is_threatened_by(target) then
        modifiers = modifiers + 0.25
    end

    -- closer targets are higher priority
    modifiers = modifiers + (20.0 - parent:dist_to_entity(target)) / 80.0

    -- weaker hostiles and stronger friendlies are higher priority
    if base == 1 then
        modifiers = modifiers + compute_offensive_strength(target_stats)
    else
        modifiers = modifiers - compute_defensive_strength(target_stats)
    end

    -- hostiles that have hurt us are higher priority
    modifiers = modifiers + parent:get_num_flag("__damage_taken_from" .. target:id())

    -- hostiles that are difficult to damage with our regular attack are lower priority
    modifiers = modifiers + parent:get_num_flag("__hard_target_for" .. target:id())

    game:debug("        Computed weight of " .. tostring(modifiers) .. " for " .. target:id())

    return base * (1 + modifiers)
end

function compute_defensive_strength(stats)
    local armor = stats.base_armor / 100.0
    local hp = stats.current_hp / 400

    return armor + hp
end

function compute_offensive_strength(stats)
    local result = 0
    if stats.caster_level > 0 then
        result = result + stats.spell_accuracy / 100.0
    else
        local damage = (stats.damage_min_0 + stats.damage_max_0) / 2.0
        result = result + damage / 100.0

        if stats.attack_is_melee then
            result = result + stats.melee_accuracy / 200.0
        else -- attack is ranged
            result = result + stats.ranged_accuracy / 200.0
        end
    end

    return result
end

-- OnDamaged script hook
function on_damaged(parent, targets, hit)
    local target = targets:first()
    local max_hp = parent:stats().max_hp
    local damage = hit:total_damage()
    local frac = damage / max_hp
    parent:add_num_flag("__damage_taken_from_" .. target:id(), frac)

    game:debug("Added damage_taken_from " .. tostring(frac) .. " for " .. target:id() .. " on "
        .. parent:id())

    -- game:log(parent:name() .. " damaged by " .. target:name() .. ": "
    --     .. hit:kind() .. " for " .. hit:total_damage() .. " damage.")
end

-- AfterAttack script hook
function after_attack(parent, targets, hit)
    local target = targets:first()
    local target_max_hp = target:stats().max_hp
    local damage = hit:total_damage()
    local frac = damage / target_max_hp

    local hard_target_factor = math.max(0.0, 0.1 - frac)
    parent:add_num_flag("__hard_target_for_" .. target:id(), hard_target_factor)

    game:debug("Added hard_target " .. tostring(hard_target_factor) .. " for " .. target:id() .. " on "
        .. parent:id())
end
