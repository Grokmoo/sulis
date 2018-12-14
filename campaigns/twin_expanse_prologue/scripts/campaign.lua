function on_rest(parent)
  game:show_confirm("Rest now?", "Yes", "No", "campaign", "fire_rest")
end

function fire_rest(parent)
  game:fade_out_in()
  game:cancel_blocking_anims()
  game:block_ui(3.0)
  game:run_script_delayed("campaign", "heal_party", 2.0)
end

function heal_party(parent)
  game:init_party_day()
  
  local party = game:party()
  for i = 1, #party do
    party[i]:remove_effects_with_tag("injury")
  end
end

function on_party_death(parent)
  parent:set_disabled(true)
  add_injury(parent)
end

function add_injury(parent)
  game:say_line("Injured!", parent)
  
  local injuries = {
    { fn = function(parent)
      local effect = parent:create_effect("Injury: Broken Leg")
	  effect:set_icon("gui/status_injury", "Injury: Broken Leg")
      effect:add_attribute_bonus("Dexterity", -6)
	  effect:add_num_bonus("movement_rate", -0.5)
      return effect
    end },
	
	{ fn = function(parent)
	  local effect = parent:create_effect("Injury: Internal Bleeding")
	  effect:set_icon("gui/status_injury", "Injury: Internal Bleeding")
	  effect:add_attribute_bonus("Endurance", -4)
	  effect:add_num_bonus("defense", -20)
	  return effect
	end },
	
	{ fn = function(parent)
	  local effect = parent:create_effect("Injury: Broken Arm")
	  effect:set_icon("gui/status_injury", "Injury: Broken Arm")
	  effect:add_attribute_bonus("Strength", -4)
	  effect:add_num_bonus("melee_accuracy", -15)
	  effect:add_num_bonus("ranged_accuracy", -15)
	  effect:add_num_bonus("spell_accuracy", -15)
	  return effect
	end },
    
    { fn = function(parent)
      local effect = parent:create_effect("Injury: Concussion")
	  effect:set_icon("gui/status_injury", "Injury: Concussion")
      effect:add_attribute_bonus("Wisdom", -4)
	  effect:add_num_bonus("fortitude", -20)
	  effect:add_num_bonus("reflex", -20)
	  effect:add_num_bonus("will", -20)
      return effect
    end },
    
    { fn = function(parent)
      local effect = parent:create_effect("Injury: Cracked Skull")
	  effect:set_icon("gui/status_injury", "Injury: Cracked Skull")
      effect:add_attribute_bonus("Intellect", -4)
	  effect:add_num_bonus("ap", -1000)
      return effect
    end },
    
    { fn = function(parent)
      local effect = parent:create_effect("Injury: Damaged Eye")
	  effect:set_icon("gui/status_injury", "Injury: Damaged Eye")
      effect:add_attribute_bonus("Perception", -4)
	  effect:add_num_bonus("crit_chance", -10)
      return effect
    end }
  }
  
  local index = math.random(#injuries)
  local effect = injuries[index].fn(parent)
  effect:set_tag("injury")
  effect:apply()
end
