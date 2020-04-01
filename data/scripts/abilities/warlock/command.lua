function on_activate(parent, ability)
  local targets = parent:targets():hostile():visible_within(ability:range())
  
  local targeter = parent:create_targeter(ability)
  targeter:set_selection_radius(ability:range())
  targeter:add_all_selectable(targets)
  targeter:add_all_effectable(targets)
  targeter:activate()
end

function on_target_select(parent, ability, targets)
  ability:activate(parent)
  game:play_sfx("sfx/confusion")

  local target = targets:first()
  
  local effect = target:create_effect(ability:name(), ability:duration())
  effect:set_tag("charm")
  
  local cb = ability:create_callback(target)
  cb:add_target(target)
  cb:set_before_attack_fn("charm_check")
  effect:add_callback(cb)
  
  target:set_flag("__command_target", parent:id())
  
  local w = target:width()
  local h = target:height()
  
  local anim = target:create_anim("charm")
  anim:set_color(anim:param(1.0),
                 anim:param(0.0),
                 anim:param(0.0),
                 anim:param(1.0))
  anim:set_moves_with_parent()
  anim:set_position(anim:param(-0.5), anim:param(-h / 2.0 - 1.0))
  anim:set_particle_size_dist(anim:fixed_dist(1.0), anim:fixed_dist(1.0))
  effect:add_anim(anim)
  effect:apply()
end

function charm_check(parent, ability, targets)
  local target = targets:first()
  
  local caster_id = parent:get_flag("__command_target")
  local caster = game:entity_with_id(caster_id)
  if caster == nil then return end
  
  local hit = caster:special_attack(parent, "Will", "Spell")
  
  local penalty = 0
  if hit:is_miss() then
    return
  elseif hit:is_graze() then
    penalty = -30
  elseif hit:is_hit() then
    penalty = -50
  elseif hit:is_crit() then
    penalty = -70
  end
  
  local gen = parent:create_anim("wind_particle", 0.5)
  gen:set_moves_with_parent()
  gen:set_initial_gen(100)
  gen:set_position(gen:param(0.0), gen:param(-1.5))
  gen:set_particle_size_dist(gen:fixed_dist(0.7), gen:fixed_dist(0.7))
  local speed = 2.0
  gen:set_particle_position_dist(gen:dist_param(gen:uniform_dist(-0.1, 0.1),
                                 gen:angular_dist(0.0, 2 * math.pi, 3.0 * speed / 4.0, speed)))
  gen:set_particle_duration_dist(gen:fixed_dist(0.5))
  gen:set_color(gen:param(1.0), gen:param(0.0), gen:param(0.0), gen:param(1.0, -2.0))
  gen:activate()
  
  local effect = parent:create_effect(ability:name(), 0)
  effect:add_num_bonus("melee_accuracy", penalty)
  effect:add_num_bonus("ranged_accuracy", penalty)
  effect:add_num_bonus("spell_accuracy", penalty)
  effect:apply()
  
  if caster:ability_level(ability) > 1 then
    local amount = penalty / -10
    parent:take_damage(caster, amount, amount, "Raw")
  end
end