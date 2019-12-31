function on_activate(parent, ability)
  local targets = parent:targets():friendly():visible_within(ability:range())
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)

  local target = targets:first()
  
  effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("haste")
  
  local stats = parent:stats()
  local amount = (2 + stats.intellect_bonus / 20) * game:ap_display_factor()
  effect:add_num_bonus("ap", amount)
  
  local gen = target:create_anim("haste")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-0.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  effect:add_anim(gen)
  effect:apply()
end
