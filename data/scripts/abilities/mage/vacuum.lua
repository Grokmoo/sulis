radius = 7.0

function on_activate(parent, ability)
  local targets = parent:targets()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_free_select(ability:range())
  targeter:set_selection_radius(ability:range())
  targeter:set_shape_circle(radius)
  targeter:invis_blocks_affected_points(true)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local position = targets:selected_point()
  
  local anim = parent:create_anim("wind_collapse", 0.7)
  anim:set_position(anim:param(position.x - 6.0), anim:param(position.y - 6.0))
  anim:set_particle_size_dist(anim:fixed_dist(12.0), anim:fixed_dist(12.0))
  anim:activate()
  
  local gen = parent:wait_anim(0.7)
  local targets = targets:to_table()
  for i = 1, #targets do
    local cb = ability:create_callback(parent)
	cb:add_target(targets[i])
	cb:add_selected_point(position)
	cb:set_on_anim_update_fn("attack_target")
    gen:add_callback(cb, 0.5 * (radius - targets[i]:dist_to_point(position)) / radius)
  end
  gen:activate()
  
  ability:activate(parent)
end

function attack_target(parent, ability, targets)
  local target = targets:first()
  
  if not target:is_valid() then return end

  local stats = parent:stats()
  local min_dmg = 15 + stats.caster_level / 3 + stats.intellect_bonus / 6
  local max_dmg = 25 + stats.intellect_bonus / 3 + stats.caster_level * 0.667
  local hit = parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 20, "Crushing")
  
  local base_dist = math.floor(8 + stats.caster_level / 3, stats.intellect_bonus / 6 - target:width())
  local point = targets:selected_point()
  local direction = -1
  
  push_target(base_dist, target, hit, point, direction)
end

--INCLUDE push_target