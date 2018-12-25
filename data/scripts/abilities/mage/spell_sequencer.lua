function on_activate(parent, ability)
  parent:clear_flag("__sequencer_spell_1")
  parent:clear_flag("__sequencer_spell_2")
  parent:clear_flag("__sequencer_type")

  cb = ability:create_callback(parent)
  cb:set_on_menu_select_fn("spell_select1")

  menu = game:create_menu("Select a Target Type", cb)
  menu:add_choice("Self", "self")
  menu:add_choice("Friendly", "friendly")
  menu:add_choice("Hostile", "hostile")
  menu:show()
end

function spell_select1(parent, ability, targets, selection)
  parent:set_flag("__sequencer_type", selection:value())

  cb = ability:create_callback(parent)
  cb:set_on_menu_select_fn("spell_select2")
  
  setup_menu(parent, ability, parent:get_flag("__sequencer_type"), cb, "Select a 1st Spell")
end

function spell_select2(parent, ability, targets, selection)
  parent:set_flag("__sequencer_spell_1", selection:value())
  
  cb = ability:create_callback(parent)
  cb:set_on_menu_select_fn("create_sequencer")
  
  setup_menu(parent, ability, parent:get_flag("__sequencer_type"), cb, "Select a 2nd Spell")
end

function setup_menu(parent, ability, target_type, cb, title)
  already_selected = "none"
  if parent:has_flag("__sequencer_spell_1") then
    already_selected = parent:get_flag("__sequencer_spell_1")
  end

  if target_type == "self" then
    abilities = { "heal", "acid_weapon", "rock_armor", "fire_shield", "haste", "air_shield", "luck", "expediate", "minor_heal" }
  elseif target_type == "friendly" then
    abilities = { "heal", "acid_weapon", "haste", "expediate", "minor_heal" }
  else
    abilities = { "flare", "acid_bomb", "crush", "frostbite", "slow", "wind_gust", "shock", "firebolt", "dazzle",
	  "flaming_fingers", "stun" }
  end

  menu = game:create_menu(title, cb)
  for i = 1, #abilities do
    ability_id = abilities[i]
	if already_selected ~= ability_id then
	  if parent:has_ability(ability_id) then
	    ability = parent:get_ability(ability_id)
	    menu:add_choice(ability:name(), ability_id)
	  end
	end
  end
  menu:show()
end

function create_sequencer(parent, ability, targets, selection)
  parent:set_flag("__sequencer_spell_2", selection:value())
  
  effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  cb = ability:create_callback(parent)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  
  gen = parent:create_anim("pulsing_particle")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(0.0), gen:param(-3.0))
  gen:set_color(gen:param(1.0), gen:param(1.0), gen:param(0.0), gen:param(1.0))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  gen:set_draw_above_entities()
  effect:add_anim(gen)
  effect:apply()
  ability:activate(parent)
  
  parent:add_ability("activate_sequencer")
end

function on_removed(parent)
  parent:clear_flag("__sequencer_spell_1")
  parent:clear_flag("__sequencer_spell_2")
  parent:clear_flag("__sequencer_type")
  
  parent:remove_ability("activate_sequencer")
end