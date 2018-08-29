function ai_action(parent, state)
  abilities = parent:abilities():can_activate():remove_kind("Special")
  abilities:sort_by_priority()

  hostiles = parent:targets():hostile()
  friendlies = parent:targets():friendly():to_table()

  if check_swap_weapons(parent, hostiles).done then
    _G.state = parent:state_wait(10)
    return
  end

  items = parent:inventory():usable_items()

  hostiles = hostiles:to_table()

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
  result = check_move_for_attack(parent, target)
  if result.attack then
    parent:anim_weapon_attack(target, nil, true)
  end

  if result.done then
    _G.state = parent:state_end()
  else
    _G.state = parent:state_wait(10)
  end
end

function check_move_for_attack(parent, target)
  if not parent:stats().attack_is_ranged then
    if parent:can_reach(target) then
      return { attack=true }
    end

    if not parent:move_towards_entity(target) then
      return { attack=false, done=true }
    end
  else
    dist = parent:dist_to_entity(target)
    target_dist = parent:vis_dist() - 2

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

--- Swaps weapon to mele is enemy is close and vice versa.
-- This will force to use ranged weapon always when possible
-- if entity has it.
function check_swap_weapons(parent, hostiles)
  has_secondary = parent:inventory():has_alt_weapons()
  if has_secondary then
    primary = parent:inventory():weapon_style()
    secondary = parent:inventory():alt_weapon_style()

    if primary == "Ranged" and secondary ~= "Ranged" and not hostiles:threatening():is_empty() then
      if parent:swap_weapons() then
        return { done=true }
      end
    elseif primary ~= "Ranged" and secondary == "Ranged" and hostiles:threatening():is_empty() then
      if parent:swap_weapons() then
        return { done=true }
      end
    end
  end

  return { done=false }

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

--- Finds the closest target.
function find_closest_target(parent, targets)
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

-- Find most wounded (less then 30% hp)
function find_wounded_target(parent, targets)
  hp_frac = 1.0
  most_wounded = nil

  for i = 1, #targets do
    stats = targets[i]:stats()
    frac = stats.current_hp / stats.max_hp
    if frac < hp_frac then
      hp_frac = frac
      most_wounded = targets[i]
    end
  end

  if hp_frac < 0.3 then
    game:log("WOUNDED: attacker: " .. parent:name() .. " enemy: " .. most_wounded:name() .. "(" .. hp_frac .. ")")
    return most_wounded
  else
    return nil
  end
end

--- Find weakest (least ammount of current_hp) 
function find_weakest_target(parent, targets)
  smallest_hp = 10000
  weakest_target = nil

  for i = 1, #targets do
    health = targets[i]:stats().current_hp
    if health < smallest_hp then
      smallest_hp = health
      weakest_target = targets[i]
    end
  end

  return weakest_target
end

--- If entity have flag "target" find enemy name set in flag and attack it if possible.
-- We can throw "else" statements out after this will setle down (no logging).
function find_and_manage_target_flag(parent, targets)
  if parent:has_flag("target") then
    for i= 1, #targets do
      if targets[i]:name() == parent:get_flag("target") then
        rounds_left = tonumber(parent:get_flag("target_rounds"))
        if rounds_left >= 1 then
          is_visible = parent:dist_to_entity(targets[i]) < parent:vis_dist()
          if targets[i]:is_valid() and is_visible then
            parent:set_flag("target_rounds", tostring(rounds_left - 1))
            game:log("TARGET FLAG: " .. parent:name() .. " remembers " .. targets[i]:name() .. " for " .. rounds_left)
            return targets[i]
          else
            game:log("TARGET FLAG: lost target: " .. targets[i]:name() .. " vis: " ..
              tostring(is_visible) .. " valid: " .. tostring(targets[i]:is_valid()))
          end
        else
          game:log("TARGET FLAG: lost interest in " .. targets[i]:name())
        end
        parent:clear_flag("target")
        parent:clear_flag("target_rounds")
        return nil
      end
    end
  end
  return nil
end

--- Find target dispatcher.
-- It decides which method use when determining target.
-- TODO: use parent perception instead of random decision.
function find_target(parent, targets)
  flagged = find_and_manage_target_flag(parent, targets)
  if flagged ~= nil then
    return flagged
  end

  wounded = find_wounded_target(parent, targets)
  if wounded ~= nil then
    game:log("found wounded!")
    return wounded
  end

  math.randomseed(os.time())
  r = math.random()

  if r <= 0.33 then 
    closest = find_closest_target(parent, targets)
    game:log("CLOSEST: attacker ".. parent:name() .. " enemy: " .. closest:name())
    return closest
  elseif (r > 0.33) and (r <= 0.44) then
    -- get random target! :-)
    math.randomseed(os.time()) 
    i = math.random(1,#targets)
    game:log("RANDOM: attacker " .. parent:name() .. " enemy: " .. targets[i]:name())
    return targets[i]
  else
    weakest = find_weakest_target(parent, targets)
    dist = parent:dist_to_entity(weakest)
    visibility_dist = parent:vis_dist() / 2
    if dist > visibility_dist then
      closest = find_closest_target(parent, targets)
      game:log("WEAKEST TOO FAR, SO CLOSEST: attacker: " .. parent:name() .. " enemy: " .. closest:name())
      return closest
    else
      game:log("WEAKEST: attacker: " .. parent:name() .. " enemy: " .. weakest:name())
      return weakest
    end
  end
end



--- OnDamaged script hook
-- TODO: determine which attack is above normal, maybe critical + some higher number?
-- Or remembering average hit in flag?
function on_damaged(parent, targets, hit)
  remember_for = 5 -- for how many attacks
  attacker = targets:first() -- there is always only actual attacker in targets

  -- game:log(parent:name() .. " damaged by " .. attacker:name() .. " : "
  --     .. hit:kind() .. " for " .. hit:total_damage() .. " damage.")
  
  if hit:total_damage() > 10 then
    game:log("TARGET FLAG: set for " .. parent:name() .. ", attacker: " .. attacker:name() .. "(" .. remember_for .. ")")
    parent:set_flag("target", attacker:name())
    parent:set_flag("target_rounds", tostring(remember_for))
  end
end
