function on_activate(parent, ability)
  local effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  
  local cb = ability:create_callback(parent)
  cb:set_on_moved_fn("on_parent_moved")
  effect:add_callback(cb)
  effect:apply()
  
  ability:activate(parent)
end

function on_parent_moved(parent, ability)
  local p = {}
  p["x"] = parent:x() + 1
  p["y"] = parent:y() + 1

  local points = {}
  points[1] = p

  fire_surface(parent, ability, points, 2)
end

--INCLUDE fire_surface