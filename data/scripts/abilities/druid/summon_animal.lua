function build_choices(parent, ability)
  local choices = { "Wolf", "Rat", "Scorpion" }
  
  local level = parent:ability_level(ability)
  
  if level > 1 then
    table.insert(choices, "Spider")
  end
  
  if level > 2 then
    table.insert(choices, "Mushroom")
  end
  
  return choices
end

function on_activate(parent, ability)
  local cb = ability:create_callback(parent)
  cb:set_on_menu_select_fn("menu_select")

  local choices = build_choices(parent, ability)
  local menu = game:create_menu("Select an animal to summon", cb)
  for i = 1, #choices do
    menu:add_choice(choices[i])
  end
  
  menu:show(parent)
end

function ai_on_activate(parent, ability)
  local choices = build_choices(parent, ability)
  local choice = choices[math.random(#choices)]
  local selection = game:create_menu_selection(choice)
  menu_select(parent, ability, nil, selection)
end

function menu_select(parent, ability, targets, selection)
  parent:set_flag("__summon_animal_type", selection:value())

  local summon_sizes = {
    Wolf = "2by2",
	Rat = "2by2",
	Scorpion = "2by2",
	Spider = "3by3",
	Mushroom = "2by2"
  }
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:set_free_select(ability:range())
  targeter:set_free_select_must_be_passable(summon_sizes[selection:value()])
  targeter:set_shape_object_size(summon_sizes[selection:value()])
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local pos = targets:selected_point()
  ability:activate(parent)
  
  local summon_type = parent:get_flag("__summon_animal_type")
  parent:clear_flag("__summon_animal_type")
  if summon_type == nil then return end
  
  local summon_ids = {
    Wolf = "wolf",
	Rat = "rat",
	Scorpion = "scorpion_medium",
	Spider = "spider_large_summon",
	Mushroom = "shroom_large_summon"
  }
  
  local summon = game:spawn_actor_at(summon_ids[summon_type], pos.x, pos.y, parent:get_faction())
  if not summon:is_valid() then return end
  
  if parent:is_party_member() then
    summon:add_to_party(false)
	summon:set_flag("__is_summoned_party_member")
  end
  
  local levels = parent:stats().caster_level
  if levels > 1 then
    summon:add_levels("fighter", levels - 1)
  end
  
  local effect = summon:create_effect(ability:name(), ability:duration())
  cb = ability:create_callback(summon)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  effect:apply()
  
  local anim = summon:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()
  
  game:play_sfx("sfx/roar5")
end

function on_removed(parent, ability)
  local cb = ability:create_callback(parent)
  cb:set_on_anim_complete_fn("on_remove_complete")

  local anim = parent:create_color_anim(1.0)
  anim:set_color(anim:param(1.0), anim:param(1.0), anim:param(1.0), anim:param(1.0, -1.0))
  anim:set_color_sec(anim:param(1.0, -1.0),
                     anim:param(1.0, -1.0),
                     anim:param(1.0, -1.0),
                     anim:param(0.0))
  anim:set_completion_callback(cb)
  anim:activate()
end

function on_remove_complete(parent, ability)
  parent:remove()
end
