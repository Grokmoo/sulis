function on_activate(parent, ability)
  local targets = parent:targets():hostile()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_free_select(ability:range())
  targeter:set_selection_radius(ability:range())
  targeter:set_shape_circle(5.0)
  targeter:invis_blocks_affected_points(true)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local anim = parent:create_anim("star_circle", 0.7)
  anim:set_position(anim:param(parent:x() - 2.0), anim:param(parent:y() - 3.0))
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(1.0))
  anim:set_draw_above_entities()
  anim:set_particle_size_dist(anim:fixed_dist(6.0), anim:fixed_dist(6.0))
  anim:activate()

  local anim = parent:wait_anim(0.6)
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("attack_targets")
  anim:set_completion_callback(cb)
  anim:activate()
  
  ability:activate(parent)
end


function attack_targets(parent, ability, targets)
  local targets = targets:to_table()
  for i = 1, #targets do
    attack_target(parent, ability, targets[i])
  end
end

function attack_target(parent, ability, target)
  if not target:is_valid() then return end
  
  local stats = parent:stats()
  local min_dmg = 8 + stats.caster_level / 3 + stats.intellect_bonus / 6
  local max_dmg = 16 + stats.intellect_bonus / 3 + stats.caster_level * 0.667
  parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 10, "Piercing")
  parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 10, "Cold")
  
  local gen = target:create_anim("ice_explode", 0.7)
  gen:set_position(gen:param(target:x()), gen:param(target:y() - 1.5))
  gen:set_particle_size_dist(gen:fixed_dist(3.0), gen:fixed_dist(3.0))
  gen:set_draw_above_entities()
  gen:activate()
  
  local anim = target:create_color_anim(1.0)
  anim:set_color_sec(anim:param(0.0, 0.0),
                     anim:param(1.0, -1.0),
                     anim:param(1.0, -1.0),
                     anim:param(0.0))
  anim:activate()
end
