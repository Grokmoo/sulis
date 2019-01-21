held_main_id = "__tree_shape_equip_held_main"
held_off_id = "__tree_shape_equip_held_off"

function on_activate(parent, ability)
  ability:activate(parent)
  
  local effect = parent:create_effect(ability:name(), ability:duration())
  effect:set_tag("shapeshift")
  
  local cb = ability:create_callback(parent)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  
  local stats = parent:stats()
  if parent:ability_level(ability) > 1 then
    local bonus = stats.caster_level + stats.wisdom_bonus
    effect:add_resistance(15 + bonus, "Slashing")
    effect:add_resistance(20 + bonus, "Piercing")
    effect:add_resistance(10 + bonus, "Crushing")
  end
  
  effect:add_attribute_bonus("Strength", 8)
  effect:add_attribute_bonus("Dexterity", -4)
  effect:add_attribute_bonus("Endurance", 8)
  effect:add_move_disabled()
  effect:add_abilities_disabled()
  
  local level = stats.caster_level / 2 + stats.wisdom_bonus / 4
 
  effect:add_num_bonus("armor", 8 + level)
  
  local inv = parent:inventory()
  local held_main = inv:unequip_item("held_main")
  local held_off = inv:unequip_item("held_off")
  
  if held_main:is_valid() then
    parent:set_flag(held_main_id, held_main:id())
  end
  
  if held_off:is_valid() then
    parent:set_flag(held_off_id, held_off:id())
  end
  
  local item = game:add_party_item("tree_attack")
  inv:equip_item(item)
  inv:set_locked(true)
  
  local gen = parent:create_image_layer_anim()
  gen:add_image("Ears", "empty")
  gen:add_image("Hair", "empty")
  gen:add_image("Beard", "empty")
  gen:add_image("Head", "empty")
  gen:add_image("Hands", "empty")
  gen:add_image("Foreground", "creatures/treeman")
  gen:add_image("Torso", "empty")
  gen:add_image("Legs", "empty")
  gen:add_image("Feet", "empty")
  gen:add_image("Background", "empty")
  effect:add_image_layer_anim(gen)
  
  effect:apply()
  
  local anim = parent:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()
end

function on_removed(parent, ability)
  local inv = parent:inventory()
  inv:set_locked(false)
  item = inv:unequip_item("held_main")
   
  if item:id() == "tree_attack" then
	game:remove_party_item(item)
  end
   
  local anim = parent:create_color_anim(1.0)
  anim:set_color_sec(anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(1.0, -1,0),
                     anim:param(0.0))
  anim:activate()
  
  if parent:has_flag(held_main_id) then
     local item_id = parent:get_flag(held_main_id)
	 local item = game:find_party_item(item_id)
	 if item:is_valid() then
	   inv:equip_item(item)
	 end
   
     parent:clear_flag(held_main_id)
   end
   
   if parent:has_flag(held_off_id) then
     local item_id = parent:get_flag(held_off_id)
	 local item = game:find_party_item(item_id)
	 if item:is_valid() then
	   inv:equip_item(item)
	 end
	 
     parent:clear_flag(held_off_id)
   end
end