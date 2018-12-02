function on_activate(parent, ability)
  ability:activate(parent)
  
  effect = parent:create_effect(ability:name(), ability:duration())
  effect:set_tag("shapeshift")
  
  cb = ability:create_callback(parent)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  
  effect:add_attribute_bonus("Strength", 3)
  effect:add_attribute_bonus("Dexterity", 2)
  effect:add_attribute_bonus("Endurance", 2)
  effect:add_abilities_disabled()
  
  stats = parent:stats()
  level = stats.caster_level / 2 + stats.wisdom_bonus / 4
 
  effect:add_num_bonus("armor", 10 + level - stats.base_armor)
  effect:add_num_bonus("defense", 50 + level * 2 - stats.defense)
  effect:add_num_bonus("melee_accuracy", 50 + level * 2 - stats.melee_accuracy)
  
  inv = parent:inventory()
  if inv:has_alt_weapons() then
    inv:unequip_item("held_main")
	inv:unequip_item("held_off")
  else
    parent:swap_weapons()
  end
  
  item = game:add_party_item("werewolf_claw")
  inv:equip_item(item)
  inv:set_locked(true)
  
  gen = parent:create_image_layer_anim()
  gen:add_image("Ears", "empty")
  gen:add_image("Hair", "empty")
  gen:add_image("Beard", "empty")
  gen:add_image("Head", "empty")
  gen:add_image("Hands", "empty")
  gen:add_image("Foreground", "creatures/werewolf")
  gen:add_image("Torso", "empty")
  gen:add_image("Legs", "empty")
  gen:add_image("Feet", "empty")
  gen:add_image("Background", "empty")
  effect:add_image_layer_anim(gen)
  
  effect:apply()
  
  anim = parent:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()
end

function on_removed(parent, ability)
  inv = parent:inventory()
  inv:set_locked(false)
  item = inv:unequip_item("held_main")
   
  if item:id() == "werewolf_claw" then
	game:remove_party_item(item)
  end
   
  anim = parent:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()
end