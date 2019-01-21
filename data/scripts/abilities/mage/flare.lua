function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible_within(8)
  
  local targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local stats = parent:stats()
  local target = targets:first()
  local min_dmg = 12 + stats.caster_level
  local max_dmg = 20 + stats.intellect_bonus / 2 + stats.caster_level
  parent:special_attack(target, "Reflex", "Spell", min_dmg, max_dmg, 0, "Fire")
  
  local gen = target:create_particle_generator("fire_particle", 0.6)
  gen:set_initial_gen(50.0)
  gen:set_position(gen:param(target:center_x()), gen:param(target:center_y()))
  gen:set_particle_size_dist(gen:fixed_dist(0.3), gen:fixed_dist(0.3))
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-5.0, 5.0)),
    gen:dist_param(gen:uniform_dist(-0.2, 0.2), gen:uniform_dist(-5.0, 5.0), gen:fixed_dist(5.0)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.6))
  gen:activate()
  
  ability:activate(parent)
end
