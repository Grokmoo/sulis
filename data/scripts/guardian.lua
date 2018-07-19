function on_activate(parent, ability)
  if not parent:inventory():has_equipped_shield() then
    game:say_line("You must have a shield equipped.", parent)
    return
  end

  targets = parent:targets():friendly():reachable():without_self()
 
  if parent:ability_level(ability) > 1 then
    targeter = parent:create_targeter(ability)
    targeter:add_selectable(parent)
    targeter:set_shape_circle(5.0)
    targeter:add_all_effectable(targets)
    targeter:activate()
  else
    targeter = parent:create_targeter(ability)
    targeter:add_all_selectable(targets)
    targeter:add_all_effectable(targets)
    targeter:activate()
  end
end

function on_target_select(parent, ability, targets)
  targets = targets:to_table()
  for i = 1, #targets do
    effect = targets[i]:create_effect(ability:name(), ability:duration())
    effect:add_num_bonus("defense", 20)
    effect:add_num_bonus("armor", 10)

    gen = targets[i]:create_anim("shield")
    gen:set_moves_with_parent()
    gen:set_position(gen:param(-0.75), gen:param(-01.5))
    gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
    effect:add_anim(gen)
    effect:apply()
  end

  ability:activate(parent)
end
