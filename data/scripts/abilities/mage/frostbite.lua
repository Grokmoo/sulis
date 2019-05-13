function on_activate(parent, ability)
  local targets = parent:targets()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(9.0)
  targeter:set_free_select(9.0)
  targeter:set_shape_object_size("3by3")
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local position = targets:selected_point()

  local anim = parent:create_anim("frostbite", 0.54)
  anim:set_position(anim:param(position.x - 2.0), anim:param(position.y - 2.0))
  anim:set_particle_size_dist(anim:fixed_dist(5.0), anim:fixed_dist(5.0))
  anim:set_alpha(anim:param(1.0))
  
  local targets = targets:to_table()
  for i = 1, #targets do
    attack_target(parent, ability, targets[i])
  end
  
  anim:activate()
  ability:activate(parent)
end

function attack_target(parent, ability, target)
  local stats = parent:stats()
  local min_dmg = 10 + stats.caster_level / 2 + stats.intellect_bonus / 4
  local max_dmg = 18 + stats.intellect_bonus / 2 + stats.caster_level
  parent:special_attack(target, "Fortitude", "Spell", min_dmg, max_dmg, 0, "Cold")
end

