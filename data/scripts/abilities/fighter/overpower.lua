function on_activate(parent, ability)
  local stats = parent:stats()
  if not stats.attack_is_melee then
    game:say_line("You must have a melee weapon equipped.", parent)
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
  cb:set_before_attack_fn("create_parent_effect")
  cb:set_after_attack_fn("create_target_effect")
  
  ability:activate(parent)
  parent:anim_weapon_attack(target, cb)
end

function create_parent_effect(parent, ability, targets)
  local target = targets:first()
  local stats = parent:stats()

  local effect = parent:create_effect(ability:name(), 0)
  
  effect:add_damage(5, 8 + stats.level / 2)
  effect:apply()
  
  local gen = target:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-1.0), gen:param(-1.0))
  gen:set_particle_size_dist(gen:fixed_dist(2.0), gen:fixed_dist(2.0))
  gen:set_color(gen:param(1.0), gen:param(0.0), gen:param(0.0))
  gen:activate()
end

function create_target_effect(parent, ability, targets, hit)
  local target = targets:first()
  
  if hit:is_miss() then return end
  
  local effect = target:create_effect(ability:name(), ability:duration())
  
  local stats = parent:stats()
  if hit:is_graze() then
    effect:add_num_bonus("armor", -5 - stats.level / 3)
	effect:add_num_bonus("defense", -10 - stats.level)
  elseif hit:is_hit() then
    effect:add_num_bonus("armor", -10 - stats.level / 2)
	effect:add_num_bonus("defense", -20 - stats.level * 1.5)
  elseif hit:is_crit() then
    effect:add_num_bonus("armor", -15 - stats.level / 2)
	effect:add_num_bonus("defense", -30 - stats.level * 2)
  end
  
  local anim = target:create_particle_generator("arrow_down")
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.37), anim:param(-1.5))
  anim:set_particle_size_dist(anim:fixed_dist(0.75), anim:fixed_dist(0.75))
  anim:set_gen_rate(anim:param(6.0))
  anim:set_initial_gen(2.0)
  anim:set_particle_position_dist(anim:dist_param(anim:uniform_dist(-0.7, 0.7), anim:uniform_dist(-0.1, 0.1)),
                                  anim:dist_param(anim:fixed_dist(0.0), anim:uniform_dist(1.0, 1.5)))
  anim:set_particle_duration_dist(anim:fixed_dist(0.75))
  anim:set_color(anim:param(1.0), anim:param(0.0), anim:param(0.0))
  effect:add_anim(anim)
  effect:apply()
end
