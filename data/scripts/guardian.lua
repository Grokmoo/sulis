function on_activate(parent, ability)
  if not parent:inventory():has_equipped_shield() then
    return
  end

  targets = parent:targets():friendly():reachable():without_self()
  
  targeter = parent:create_targeter(ability)
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  target = targets:first()
  
  effect = target:create_effect(ability:name(), ability:duration())
  effect:add_num_bonus("defense", 20)
  effect:add_num_bonus("armor", 10)

  gen = target:create_anim("shield")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-01.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:set_color(gen:param(0.1), gen:param(0.6), gen:param(0.6))
  effect:apply(gen)

  ability:activate(parent)
end
