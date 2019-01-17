function on_activate(parent, ability)
  cb = ability:create_callback(parent)
  cb:set_on_menu_select_fn("menu_select")

  level = parent:ability_level(ability)
  
  menu = game:create_menu("Select an animal to summon", cb)
  menu:add_choice("Wolf")
  menu:add_choice("Rat")
  menu:add_choice("Scorpion")
  if level > 1 then
    menu:add_choice("Spider")
  end
  
  if level > 2 then
    menu:add_choice("Mushroom")
  end
  
  menu:show()
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
  
  targeter = parent:create_targeter(ability)
  targeter:set_free_select(12.0)
  targeter:set_free_select_must_be_passable(summon_sizes[selection:value()])
  targeter:set_shape_object_size(summon_sizes[selection:value()])
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  pos = targets:selected_point()
  ability:activate(parent)
  
  summon_type = parent:get_flag("__summon_animal_type")
  parent:clear_flag("__summon_animal_type")
  if summon_type == nil then return end
  
  local summon_ids = {
    Wolf = "wolf",
	Rat = "rat",
	Scorpion = "scorpion_medium",
	Spider = "spider_large_summon",
	Mushroom = "shroom_large_summon"
  }
  
  summon = game:spawn_actor_at(summon_ids[summon_type], pos.x, pos.y, "Friendly")
  if not summon:is_valid() then return end
  
  summon:add_to_party(false)
  summon:set_flag("__is_summoned_party_member")
  
  levels = parent:stats().caster_level
  if levels > 1 then
    summon:add_levels("fighter", levels - 1)
  end
  
  effect = summon:create_effect(ability:name(), ability:duration())
  cb = ability:create_callback(summon)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  effect:apply()
  
  anim = summon:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()
end

function on_removed(parent, ability)
  cb = ability:create_callback(parent)
  cb:set_on_anim_complete_fn("on_remove_complete")

  anim = parent:create_color_anim(1.0)
  anim:set_color(anim:param(1.0), anim:param(1.0), anim:param(1.0), anim:param(1.0, -1.0))
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:set_completion_callback(cb)
  anim:activate()
end

function on_remove_complete(parent, ability)
  parent:remove()
end