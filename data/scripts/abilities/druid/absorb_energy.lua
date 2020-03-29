choices = { "Piercing", "Slashing", "Crushing" }

function on_activate(parent, ability)
  local cb = ability:create_callback(parent)
  cb:set_on_menu_select_fn("menu_select")

  local menu = game:create_menu("Select type for Absorb Energy", cb)
  for i = 1, #choices do
    menu:add_choice(choices[i])
  end
  menu:show(parent)
end

function ai_on_activate(parent, ability)
  local choice = choices[math.random(#choices)]
  local selection = game:create_menu_selection(choice)
  menu_select(parent, ability, nil, selection)
end

function menu_select(parent, ability, targets, selection)
  local effect = parent:create_effect(ability:name(), ability:duration())
  effect:set_tag("magic_defense")
  
  local w = parent:width() + 2
  local h = parent:height() + 2

  parent:set_flag("__absorb_energy_type", selection:value())

  local cb = ability:create_callback(parent)
  cb:set_on_damaged_fn("on_damaged")
  cb:set_on_removed_fn("on_removed")
  effect:add_callback(cb)
  
  local gen = parent:create_anim("shell")
  gen:set_moves_with_parent()
  gen:set_position(gen:param(-w / 2.0), gen:param(-h / 2.0 - 1.0))
  gen:set_particle_size_dist(gen:fixed_dist(w), gen:fixed_dist(h))
  gen:set_color(gen:param(0.0), gen:param(1.0), gen:param(1.0))
  effect:add_anim(gen)
  effect:apply()
  
  ability:activate(parent)
  game:play_sfx("sfx/blessing3")
end

function on_damaged(parent, ability, targets, hit)
  local attacker = targets:first()
  local defender = parent
  
  -- game:log(attacker:name() .. " hit " .. defender:name() .. " with type " .. hit:kind() .. " for " .. hit:total_damage())
  -- entries = hit:entries()
  -- for i = 1, #entries do
  --   game:log("Type: " .. entries[i]:kind() .. ", amount: " .. tostring(entries[i]:amount()))
  -- end
  
  local damage_type = parent:get_flag("__absorb_energy_type")
  if damage_type == nil then return end
  
  local matching_damage = hit:damage_of_type(damage_type)
  if matching_damage == 0 then return end
  
  local stats = parent:stats()
  local amount = 25 + stats.caster_level + stats.wisdom_bonus
  local heal = matching_damage * (1.0 + amount / 100.0)
  defender:heal_damage(heal)
  game:play_sfx("sfx/blessing3")
end

function on_removed(parent)
  parent:clear_flag("__absorb_energy_type")
end
