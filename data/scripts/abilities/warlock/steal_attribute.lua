choices = { "Strength", "Dexterity", "Endurance", "Perception", "Intellect", "Wisdom" }

function on_activate(parent, ability)
  local cb = ability:create_callback(parent)
  cb:set_on_menu_select_fn("menu_select")

  local level = parent:ability_level(ability)
  
  local menu = game:create_menu("Select an attribute to steal", cb)
  for i = 1, #choices do
    menu:add_choice(choices[i])
  end
  
  menu:show(parent)
end

function ai_on_activate(parent, ability)
  local choice = choices[math.random(#choices)]
  local selection = game:create_menu_selection(choice)
  menu_select(parent, ability, nil, selection)
end

function menu_select(parent, ability, targets, selection)
  parent:set_flag("__steal_attribute_type", selection:value())
  
  local targets = parent:targets():hostile():touchable()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_touchable()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  ability:activate(parent)
  
  local attr = parent:get_flag("__steal_attribute_type")
  parent:clear_flag("__steal_attribute_type")
  if attr == nil then return end
  
  local anim = target:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1.0),
                     anim:param(0.0),
                     anim:param(0.0),
                     anim:param(0.0))
  anim:activate()
  
  local hit = parent:special_attack(target, "Will", "Spell")
  local duration = ability:duration()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    duration = duration - 1
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = duration + 1
  end
  
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("weaken")
  
  local stats = parent:stats()
  local amount = 2 + stats.intellect_bonus / 4 + stats.caster_level / 4
  effect:add_attribute_bonus(attr, -amount)
  
  local anim = target:create_particle_generator("arrow_down")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  anim:set_gen_rate(anim:param(1.0))
  anim:set_initial_gen(1.0)
  anim:set_particle_position_dist(anim:dist_param(anim:fixed_dist(0.0)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:fixed_dist(1.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0), anim:param(0.8))
  effect:add_anim(anim)
  effect:apply()
  
  local effect = parent:create_effect(ability:name(), ability:duration())
  effect:add_attribute_bonus(attr, amount)
  
  if parent:ability_level(ability) > 1 then
    if attr == "Strength" then
	  effect:add_attribute_bonus("Dexterity", amount / 2)
	  effect:add_attribute_bonus("Endurance", amount / 2)
	elseif attr == "Dexterity" then
	  effect:add_attribute_bonus("Perception", amount / 2)
	  effect:add_attribute_bonus("Endurance", amount / 2)
	elseif attr == "Endurance" then
	  effect:add_attribute_bonus("Strength", amount / 2)
	  effect:add_attribute_bonus("Dexterity", amount / 2)
	elseif attr == "Perception" then
	  effect:add_attribute_bonus("Intellect", amount / 2)
	  effect:add_attribute_bonus("Dexterity", amount / 2)
	elseif attr == "Intellect" then
	  effect:add_attribute_bonus("Perception", amount / 2)
	  effect:add_attribute_bonus("Wisdom", amount / 2)
	elseif attr == "Wisdom" then
	  effect:add_attribute_bonus("Perception", amount / 2)
	  effect:add_attribute_bonus("Intellect", amount / 2)
	end
  end
  
  local anim = parent:create_particle_generator("arrow_up")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  anim:set_gen_rate(anim:param(1.0))
  anim:set_initial_gen(1.0)
  anim:set_particle_position_dist(anim:dist_param(anim:fixed_dist(0.0)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:fixed_dist(-1.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(0.0), anim:param(0.8))
  effect:add_anim(anim)
  effect:apply()
end
