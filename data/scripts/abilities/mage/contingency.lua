function on_activate(parent, ability)
  local cb = ability:create_callback(parent)
  cb:set_on_menu_select_fn("spell_select")

  local menu = game:create_menu("Select a Condition on the Caster", cb)
  menu:add_choice("Movement Disabled", "move_disabled")
  menu:add_choice("Damaged", "damaged")
  menu:add_choice("Hit points below 50%", "hit_points_50")
  menu:add_choice("Hit points below 25%", "hit_points_25")
  menu:add_choice("Attacked", "attacked")
  menu:add_choice("Attacked in Melee", "attacked_melee")
  menu:show(parent)
end

function on_deactivate(parent, ability)
  ability:deactivate(parent)
end

function spell_select(parent, ability, targets, selection)
  parent:set_flag("__contingency_trigger_condition", selection:value())

  local cb = ability:create_callback(parent)
  cb:set_on_menu_select_fn("create_contingency")
  
  local abilities = { "heal", "acid_weapon", "rock_armor", "fire_shield", "haste", "air_shield", "luck", "expediate", "minor_heal" }
  
  local menu = game:create_menu("Select a Spell", cb)
  for i = 1, #abilities do
    local ability_id = abilities[i]
	if parent:has_ability(ability_id) then
	  local ability = parent:get_ability(ability_id)
	  menu:add_choice(ability:name(), ability_id)
	end
  end
  menu:show(parent)
end

function create_contingency(parent, ability, targets, selection)
  parent:set_flag("__contingency_spell", selection:value())
  
  local effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  local cb = ability:create_callback(parent)
  cb:set_on_damaged_fn("on_damaged")
  cb:set_on_effect_applied_fn("on_effect_applied")
  cb:set_after_defense_fn("after_defense")
  effect:add_callback(cb)
  
  local anim = parent:create_anim("time_spin")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-1.5), anim:param(-0.4))
  anim:set_particle_size_dist(anim:fixed_dist(3.0), anim:fixed_dist(1.5))
  anim:set_draw_below_entities()
  effect:add_anim(anim)

  effect:apply()
  ability:activate(parent)
  
  game:play_sfx("sfx/echo02")
end

function fire_contingency(parent, ability)
  local selection = parent:get_flag("__contingency_spell")

  ability:deactivate(parent)
  game:say_line("Contingency", parent)
  
  local effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("ability_ap_cost", 10 * game:ap_display_factor())
  effect:add_free_ability_group_use()
  effect:apply()
  
  local ability_to_fire = parent:get_ability(selection)
  parent:use_ability(ability_to_fire, true)
  
  if game:has_targeter() then
    if game:check_targeter_position(parent:x(), parent:y()) then
	  game:activate_targeter()
	else
	  game:cancel_targeter()
	  game:warn("Unable to fire contingency for " .. selection)
	end
  end
  
  game:play_sfx("sfx/echo02")
end

function on_effect_applied(parent, ability, targets, effect)
  if get_trigger_condition(parent) == "move_disabled" then
    if effect:has_bonus_of_kind("move_disabled") then
      fire_contingency(parent, ability)
    end
  end
end

function after_defense(parent, ability, targets, hit)
  local attacker = targets:first()
  local defender = parent
  
  if get_trigger_condition(parent) == "attacked_melee" then
    if attacker:inventory():weapon_style() ~= "Ranged" then
	  fire_contingency(parent, ability)
	end
  elseif get_trigger_condition(parent) == "attacked" then
    fire_contingency(parent, ability)
  end
end

function on_damaged(parent, ability, targets, hit)
  local stats = parent:stats()
  local hp = stats.current_hp
  local max_hp = stats.max_hp
  
  if get_trigger_condition(parent) == "damaged" then
    if hp < max_hp then
	  fire_contingency(parent, ability)
	end
  elseif get_trigger_condition(parent) == "hit_points_50" then
    if hp < max_hp / 2 then
	  fire_contingency(parent, ability)
	end
  elseif get_trigger_condition(parent) == "hit_points_25" then
    if hp < max_hp / 4 then
	  fire_contingency(parent, ability)
	end
  end
end

function get_trigger_condition(parent)
  return parent:get_flag("__contingency_trigger_condition")
end

function on_removed(parent)
  parent:clear_flag("__contingency_trigger_condition")
  parent:clear_flag("__contingency_spell")
end
