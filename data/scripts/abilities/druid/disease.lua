function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible_within(10)
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(10.0)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  local stats = parent:stats()
  local target = targets:first()
  
  local hit = parent:special_attack(target, "Fortitude", "Spell")
  local factor = 0
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    factor = 1
  elseif hit:is_hit() then
    factor = 2
  elseif hit:is_crit() then
    factor = 3
  end
  
  local amount = (1 + stats.caster_level / 8 + stats.wisdom_bonus / 8) * factor
  
  local effect = target:create_effect(ability:name())
  effect:set_tag("disease")
  effect:set_icon("gui/status_disease", "Disease")
  
  effect:add_attribute_bonus("Strength", -amount)
  effect:add_attribute_bonus("Dexterity", -amount)
  effect:add_attribute_bonus("Endurance", -amount)
  effect:add_attribute_bonus("Perception", -amount)
  
  local anim = target:create_particle_generator("heal")
  anim:set_moves_with_parent()
  anim:set_color(anim:param(1.0), anim:param(1.0), anim:param(0.0))
  anim:set_position(anim:param(-0.5), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  anim:set_gen_rate(anim:param(3.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(1.0, 1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  effect:add_anim(anim)
  
  effect:apply()
end
