function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible_within(ability:range())
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  ability:activate(parent)
  
  local hit = parent:special_attack(target, "Will", "Spell")
  local duration = ability:duration()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    duration = duration - 1
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = duration + 1
  end
  
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("charm")
  
  local anim = target:create_particle_generator("particles/heart19")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.5), anim:fixed_dist(0.5))
  anim:set_gen_rate(anim:param(3.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.3, 0.3)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(-1.0, -1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(1.2))
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0), anim:param(0.7))
  effect:add_anim(anim)
  
  local cb = ability:create_callback(target)
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  
  local faction = target:get_faction()
  target:set_flag("__charm_original_faction", faction)
  
  if faction == "Hostile" then
    target:set_faction("Friendly")
  elseif faction == "Friendly" then
    target:set_faction("Hostile")
  end
  
  effect:apply()
end

function on_removed(parent, ability)
  parent:set_faction(parent:get_flag("__charm_original_faction"))
end