--INCLUDE array_retain

function on_activate(parent, ability)
  local targets = parent:targets():hostile():touchable()
  
  -- only allow targets that are smaller than the parent
  local targets = targets:to_table()
  array_retain(
    targets,
    function(targets, i)
      return targets[i]:width() < parent:width()
    end
  )
  local targets = parent:targets_from(targets)
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_touchable()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_after_attack_fn("create_grab_effect")
  
  ability:activate(parent)
  
  local accuracy = 10 + 5 * parent:width() - 5 * target:width()
  if parent:ability_level(ability) > 1 then
    accuracy = accuracy + 20
  end
  
  local effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("melee_accuracy", accuracy)
  effect:apply()
  
  parent:anim_special_attack(target, "Fortitude", "Melee", 0, 0, 0, "Raw", cb)
end

function create_grab_effect(parent, ability, targets, hit)
  local target = targets:first()
  
  if not target:is_valid() then return end
  if hit:is_miss() then return end
  
  local duration = ability:duration()
  if hit:is_graze() then
    duration = duration - 1
  end
  
  local effect = target:create_effect(ability:name(), duration)
  
  effect:add_move_disabled()
  effect:add_attack_disabled()
  effect:add_abilities_disabled()
    
  local gen = target:create_anim("imprison")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.74), gen:param(-1.0))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:set_color(gen:param(1.0), gen:param(0.2), gen:param(0.0))
  effect:add_anim(gen)
  
  
  local delta_x = parent:x() - target:x() - 0.75
  local delta_y = parent:y() - target:y() - 0.75
  
  local gen = target:create_subpos_anim()
  gen:set_position(gen:param(delta_x), gen:param(delta_y))
  effect:add_subpos_anim(gen)
  effect:apply()
  
  local parent_effect = parent:create_effect(ability:name(), duration)
  parent_effect:add_move_disabled()
  parent_effect:apply()
end
