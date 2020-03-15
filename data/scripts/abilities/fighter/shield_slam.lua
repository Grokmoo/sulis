function on_activate(parent, ability)
  if not parent:inventory():has_equipped_shield() then
    game:say_line("You must have a shield equipped.", parent)
    return
  end

  local targets = parent:targets():hostile():attackable()
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_attackable()
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  local target = targets:first()
  
  local cb = ability:create_callback(parent)
  cb:add_target(target)
  cb:set_after_attack_fn("create_stun_effect")
 
  if parent:ability_level(ability) > 1 then
    local effect = parent:create_effect(ability:name(), 0)
    effect:add_num_bonus("melee_accuracy", 25 + parent:stats().level)
    effect:apply()
  end
  
  ability:activate(parent)
  parent:anim_special_attack(target, "Fortitude", "Melee", 0, 0, 0, "Raw", cb)
end

function create_stun_effect(parent, ability, targets, hit)
  if hit:is_miss() then
    game:play_sfx("sfx/swish_2")
    return
  end

  game:play_sfx("sfx/metal_hit_01")

  local target = targets:first()
  
  -- compute the max target pushback distance
  local pushback_dist = 2 + parent:width() - target:width()
  if parent:ability_level(ability) > 1 then
    pushback_dist = pushback_dist + 3
  end
  
  local point = {x = parent:x(), y = parent:y()}
  local direction = 1
  
  local gen = target:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-0.75))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:activate()
  
  push_target(pushback_dist, target, hit, point, direction)
end

--INCLUDE push_target
