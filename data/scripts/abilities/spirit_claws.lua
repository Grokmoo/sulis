function on_activate(parent, ability)
  ability:activate(parent)
  
  effect = parent:create_effect(ability:name(), ability:duration())
  effect:set_tag("shapeshift")
  
  cb = ability:create_callback(parent)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  
  effect:add_num_bonus("spell_accuracy", -30)
  
  inv = parent:inventory()
  if inv:has_alt_weapons() then
    inv:unequip_item("held_main")
	inv:unequip_item("held_off")
  else
    parent:swap_weapons()
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
  
  effect:apply()
end

function on_removed(parent, ability)
   inv = parent:inventory()
   inv:set_locked(false)
   item = inv:unequip_item("held_main")
   
   if item:id() == "spirit_claw" then
	game:remove_party_item(item)
   end
end