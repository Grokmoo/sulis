function on_activate(parent, ability)
  local cur_mode = parent:get_active_mode()
  if cur_mode ~= nil then
    cur_mode:deactivate(parent)
  end

  local stats = parent:stats()
  local amount = 5 + stats.level / 2
  
  local effect = parent:create_effect(ability:name())
  effect:deactivate_with(ability)
  effect:add_num_bonus("defense", amount)
  effect:add_num_bonus("reflex", amount / 2)
  effect:add_num_bonus("fortitude", amount / 2)
  effect:add_num_bonus("crit_chance", -6)
  effect:add_num_bonus("crit_multiplier", -0.5)
  effect:add_num_bonus("melee_accuracy", -10)

  local cb = ability:create_callback(parent)
  cb:set_on_held_changed_fn("on_held_changed")

  if parent:ability_level(ability) > 1 then
    cb:set_after_defense_fn("after_defense")
  end
  
  effect:add_callback(cb)

  local gen = parent:create_anim("shield")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-0.5), gen:param(-2.5))
  gen:set_particle_size_dist(gen:fixed_dist(1.0), gen:fixed_dist(1.0))
  effect:add_anim(gen)
  effect:apply()

  ability:activate(parent)
  
  game:play_sfx("sfx/metal_01")
end

function on_deactivate(parent, ability)
  ability:deactivate(parent)
end

function on_held_changed(parent, ability)
  if not parent:inventory():has_equipped_shield() then
    game:say_line("Defensive Fighting Deactivated", parent)
    ability:deactivate(parent)
  end
end

function after_defense(parent, ability, targets, hit)
  if hit:total_damage() < 2 then return end
  
  local target = targets:first()

  if not target:stats().attack_is_melee then return end

  local max_damage = math.floor(hit:total_damage() * 0.3)
  
  target:take_damage(parent, max_damage, max_damage, "Raw")
end
