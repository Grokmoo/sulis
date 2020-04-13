function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible_within(ability:range())
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  local stats = parent:stats()
  local target = targets:first()
  
  local hit = parent:special_attack(target, "Reflex", "Spell")
  local duration = ability:duration()
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    duration = duration / 2
  elseif hit:is_hit() then
    -- do nothing
  elseif hit:is_crit() then
    duration = duration * 3 / 2
  end
  
  apply_damage(parent, ability, targets)
  
  local effect = target:create_effect(ability:name(), duration)
  effect:set_tag("weaken")
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_on_round_elapsed_fn("apply_damage")
  effect:add_callback(cb)
  
  local anim = target:create_particle_generator("particles/circle4")
  anim:set_moves_with_parent()
  anim:set_initial_gen(10.0)
  anim:set_color(anim:param(0.0), anim:param(1.0), anim:param(0.2))
  anim:set_gen_rate(anim:param(20.0))
  anim:set_position(anim:param(0.0), anim:param(-0.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.3), anim:fixed_dist(0.3))
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.5, 0.5), anim:uniform_dist(-1.0, 1.0)),
    anim:dist_param(anim:uniform_dist(-0.2, 0.2), anim:uniform_dist(-1.0, 1.0), anim:fixed_dist(5.0)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.3))
  effect:add_anim(anim)
  effect:apply()
end

function apply_damage(parent, ability, targets)
  local stats = parent:stats()
  local target = targets:first()
  if target:is_dead() then return end
  
  local max_dmg = 8 + stats.caster_level / 2
  target:take_damage(parent, 4, max_dmg, "Acid", 5)
  game:play_sfx("sfx/splash")
end
