max_dist = 30

function on_activate(parent, ability)
  local stats = parent:stats()
  if not stats.attack_is_ranged then
    game:say_line("You must have a ranged weapon equipped.", parent)
    return
  end

  local targets = parent:targets():without_self()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_visible()
  targeter:set_free_select(max_dist * 2)
  targeter:set_shape_cone(parent:center_x(), parent:center_y(), 1.0, max_dist, math.pi / 8) 
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local targets = targets:to_table()
  for i = 1, #targets do
    parent:anim_weapon_attack(targets[i])
  end

  ability:activate(parent)
end
