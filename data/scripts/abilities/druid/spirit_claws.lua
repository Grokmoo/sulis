held_main_id = "__spirit_claws_equip_held_main"
held_off_id = "__spirit_claws_equip_held_off"

function on_activate(parent, ability)
  ability:activate(parent)
  
  effect = parent:create_effect(ability:name(), ability:duration())
  effect:set_tag("shapeshift")
  
  cb = ability:create_callback(parent)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  
  effect:add_num_bonus("spell_accuracy", -30)
  
  inv = parent:inventory()
  
  held_main = inv:unequip_item("held_main")
  held_off = inv:unequip_item("held_off")
  
  if held_main:is_valid() then
    parent:set_flag(held_main_id, held_main:id())
  end
  
  if held_off:is_valid() then
    parent:set_flag(held_off_id, held_off:id())
  end
  
  item = game:add_party_item("spirit_claw")
  inv:equip_item(item)
  inv:set_locked(true)
  
  gen = parent:create_anim("spirit_claw")
  gen:set_moves_with_parent()
  
  offset = parent:image_layer_offset("Hands")
  x = offset.x - math.floor(parent:width() / 2.0)
  y = offset.y - math.floor(parent:height() / 2.0)
  
  gen:set_position(gen:param(x), gen:param(y))
  gen:set_particle_size_dist(gen:fixed_dist(3.0), gen:fixed_dist(3.0))
  effect:add_anim(gen)
  effect:apply()
end

function on_removed(parent, ability)
   inv = parent:inventory()
   inv:set_locked(false)
   item = inv:unequip_item("held_main")
   
   if item:id() == "spirit_claw" then
	game:remove_party_item(item)
   end
   
   if parent:has_flag(held_main_id) then
     item_id = parent:get_flag(held_main_id)
	 item = game:find_party_item(item_id)
	 if item:is_valid() then
	   inv:equip_item(item)
	 end
   
     parent:clear_flag(held_main_id)
   end
   
   if parent:has_flag(held_off_id) then
     item_id = parent:get_flag(held_off_id)
	 item = game:find_party_item(item_id)
	 if item:is_valid() then
	   inv:equip_item(item)
	 end
	 
     parent:clear_flag(held_off_id)
   end
end