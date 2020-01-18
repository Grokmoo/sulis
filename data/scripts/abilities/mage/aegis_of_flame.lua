function on_activate(parent, ability)
  local targets = parent:targets():friendly()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:add_selectable(parent)
  targeter:set_shape_circle(ability:range())
  targeter:add_all_effectable(targets)
  targeter:invis_blocks_affected_points(true)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local anim = parent:create_anim("star_circle", 0.7)
  anim:set_position(anim:param(parent:x() - 3.0), anim:param(parent:y() - 4.0))
  anim:set_color(anim:param(1.0), anim:param(0.5), anim:param(0.0))
  anim:set_draw_above_entities()
  anim:set_particle_size_dist(anim:fixed_dist(8.0), anim:fixed_dist(8.0))
  anim:activate()

  local anim = parent:wait_anim(0.5)
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("create_fire_surface")
  anim:set_completion_callback(cb)
  anim:activate()
  
  ability:activate(parent)
end

function create_fire_surface(parent, ability, targets)
  local targets_table = targets:to_table()
  for i = 1, #targets_table do
    create_aegis_effect(parent, ability, targets_table[i])
  end

  local points = targets:random_affected_points(0.7)
  
  fire_surface(parent, ability, points, 2)
end

function create_aegis_effect(parent, ability, target)
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:add_resistance(100, "Fire")
  
  local stats = parent:stats()
  local min_dmg = 2 + stats.caster_level / 4 + stats.intellect_bonus / 6
  local max_dmg = 4 + stats.intellect_bonus / 3 + stats.caster_level / 2
  effect:add_damage_of_kind(min_dmg, max_dmg, "Fire")
  
  local anim = target:create_anim("spin_slash")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-2.0), anim:param(-3.0))
  anim:set_draw_above_entities()
  anim:set_particle_size_dist(anim:fixed_dist(4.0), anim:fixed_dist(4.0))
  effect:add_anim(anim)
  
  local cb = ability:create_callback(target)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  target:add_ability("flaming_bolt")
  
  effect:apply()
end

-- Aegis Effect on remove
function on_removed(parent)
  parent:remove_ability("flaming_bolt")
end

--INCLUDE fire_surface
