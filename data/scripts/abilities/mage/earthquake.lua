radius = 9.0

function on_activate(parent, ability)
  local targets = parent:targets()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_free_select(15.0)
  targeter:set_selection_radius(15.0)
  targeter:set_shape_circle(radius)
  targeter:invis_blocks_affected_points(true)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local anim = parent:wait_anim(0.5)
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_update_fn("create_random_anim")
  anim:add_callback(cb, 0.1)
  anim:add_callback(cb, 0.2)
  anim:add_callback(cb, 0.3)
  anim:add_callback(cb, 0.4)
  anim:add_callback(cb, 0.5)
  anim:activate()
  
  create_random_anim(parent, ability, targets)
  
  local anim = parent:wait_anim(1.0)
  local cb = ability:create_callback(parent)
  cb:add_targets(targets)
  cb:set_on_anim_complete_fn("attack_targets")
  anim:set_completion_callback(cb)
  anim:activate()
  
  ability:activate(parent)
end

function create_random_anim(parent, ability, targets)
  local ids = { "01", "02", "03" }
  local id = ids[math.random(#ids)]
  local id = "01"

  local position = targets:selected_point()

  local anim = parent:create_anim("shooting_bolt" .. id, 0.7)
  anim:set_position(
    anim:param(position.x - radius - 1.0 + (math.random() - 0.5) * 4.0),
    anim:param(position.y - 1.0 + (math.random() - 0.5) * 4.0)
  )
  anim:set_particle_size_dist(anim:fixed_dist(20.0), anim:fixed_dist(2.5))
  anim:set_rotation(anim:param(math.random() * 2.0 * math.pi))
  anim:set_color(anim:param(0.53), anim:param(0.46), anim:param(0.40))
  anim:activate()
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
  local min_dmg = 15 + stats.caster_level / 3 + stats.intellect_bonus / 6
  local max_dmg = 25 + stats.intellect_bonus / 3 + stats.caster_level * 0.667
  local hit = parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 0, "Crushing")
  
  local amount = -(5 + stats.intellect_bonus / 10) * game:ap_display_factor()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    amount = amount / 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    amount = amount * 1.5
  end
  
  target:change_overflow_ap(amount)
  
  local gen = target:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-0.75))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:activate()
end
