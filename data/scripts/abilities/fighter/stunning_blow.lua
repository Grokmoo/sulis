function on_activate(parent, ability)
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
  
  ability:activate(parent)
  parent:anim_special_attack(target, "Fortitude", "Melee", 0, 0, 0, "Raw", cb)
end

function create_stun_effect(parent, ability, targets, hit)
  local target = targets:first()
  
  if hit:is_miss() then
    game:play_sfx("sfx/swish_2")
  elseif hit:is_graze() then
    game:play_sfx("sfx/thwack-07")
    target:change_overflow_ap(-2000)
  elseif hit:is_hit() then
    target:change_overflow_ap(-4000)
	game:play_sfx("sfx/thwack-08")
  elseif hit:is_crit() then
    game:play_sfx("sfx/thwack-09")
    target:change_overflow_ap(-6000)
  end
  
  local gen = target:create_anim("burst", 0.15)
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.75), gen:param(-0.75))
  gen:set_particle_size_dist(gen:fixed_dist(1.5), gen:fixed_dist(1.5))
  gen:activate()
end
