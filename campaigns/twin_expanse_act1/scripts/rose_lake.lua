function tour_guide(parent)
  game:cancel_blocking_anims()
  game:scroll_view(14, 113)
  local guide = game:entity_with_id("rose_lake_guide")
  game:start_conversation("rose_lake_tour_guide", guide)
end

function council_secretary_suggestion(parent)
  game:set_quest_entry_state("seeing_the_council", "blocked", "Visible")
end

function weasel_end(parent)
  local weasel = game:entity_with_id("rose_lake_weasel")
  local coins = math.floor(weasel:get_num_flag("coins_to_take"))
  if coins > 0 then
    game:add_party_coins(coins)
  end
end

function weasel_debt_take_complete(parent)
  local jevero = game:entity_with_id("rose_lake_q01")
  
  game:set_quest_entry_state("a_weasels_debt", "take_complete", "Visible")
  game:set_quest_state("a_weasels_debt", "Complete")
  
  local percent_fee = (1 + math.floor(jevero:get_num_flag("negotiate"))) * 10
  
  local coins = math.floor(jevero:get_num_flag("coins_to_take"))
  if coins > 0 then
    game:add_party_coins(-coins)
	if percent_fee > 0 then
	  game:add_party_coins(coins * percent_fee / 100)
	end
  end
end

function weasel_debt_help_complete(parent)
  game:add_party_xp(100)
  
  game:set_quest_entry_state("a_weasels_debt", "help_complete", "Visible")
  game:set_quest_state("a_weasels_debt", "Complete")
end

function enable_naathfir(parent)
  game:set_world_map_location_visible("naathfir_road", true)
  game:set_world_map_location_enabled("naathfir_road", true)
end

function guard_lieutenant(parent)
  game:set_world_map_location_visible("lake_grounds", true)
  game:set_world_map_location_enabled("lake_grounds", true)
end

function arzel_fight_init(parent)
  game:cancel_blocking_anims()
  game:scroll_view(107, 15)
  
  local arzel = game:entity_with_id("arzel")
  arzel:set_faction("Hostile")
  
  game:start_conversation("arzel", arzel)
end

function arzel_spawn(parent)
  game:spawn_encounter_at(92, 15)
  game:set_quest_entry_state("seeing_the_council", "arzel", "Visible")
end

function guard_quest_complete(parent)
  game:set_quest_entry_state("seeing_the_council", "granted", "Visible")
  game:add_party_xp(200)
end

function open_council(parent, target)
  if not target:has_flag("rose_lake_council_door_opened") then
    game:enable_prop_at(36, 28)
	game:toggle_prop_at(36, 28)
	target:set_flag("rose_lake_council_door_opened")
  end
end

function remove_staff(parent)
  local item = game:find_party_item("aegis_staff")
  game:remove_party_item(item)
  
  game:set_quest_entry_state("seeing_the_council", "complete", "Visible")
  game:set_quest_state("seeing_the_council", "Complete")
  
  game:set_world_map_location_visible("naathfir_road", true)
  game:set_world_map_location_enabled("naathfir_road", true)
end

function dwarven_goods_complete(parent)
  game:set_quest_entry_state("dwarven_goods", "complete", "Visible")
  game:set_quest_state("dwarven_goods", "Complete")
  
  local weapons = { "dwarven_battleaxe", "dwarven_crossbow", "dwarven_greataxe",
    "dwarven_greathammer", "dwarven_halberd", "dwarven_longspear" }

  local armor = { "dwarven_torso_plate", "dwarven_shield", "dwarven_legs_plate" }
  
  game:add_party_item(weapons[math.random(#weapons)], "fine")
  game:add_party_item(armor[math.random(#armor)], "fine")
  game:add_party_xp(200)
  game:add_party_coins(2000)
end

function rose_fort_final_boss(parent)
  game:block_ui(4.0)
  game:fade_out_in()
  game:cancel_blocking_anims()
  game:run_script_delayed("rose_lake", "rose_fort_final_boss2", 2.0)
end

function rose_fort_final_boss2(parent)
  game:toggle_prop_at(36, 28)
  game:disable_prop_at(36, 28)
  game:scroll_view(29, 13)
  game:transition_party_to(29, 13)
  game:run_script_delayed("rose_lake", "rose_fort_final_boss3", 2.0)
end

function rose_fort_final_boss3(parent)
  local target = game:entity_with_id("rose_lake_cc0")
  game:start_conversation("rose_lake_berkeley_boss", target)
end

function rose_fort_kill_council(parent)
  local attacker = game:entity_with_id("rose_lake_cc0")

  local target = game:entity_with_id("rose_lake_cc1")
  target:take_damage(attacker, 500, 500, "Raw")
  
  target = game:entity_with_id("rose_lake_cc2")
  target:take_damage(attacker, 500, 500, "Raw")
  
  target = game:entity_with_id("rose_lake_cc3")
  target:take_damage(attacker, 500, 500, "Raw")
end

function rose_fort_portal(parent)
  local target = game:entity_with_id("rose_lake_cc0")
  
  local anim = target:create_anim("teleport")
  anim:set_position(anim:param(30),
                    anim:param(14))
  anim:set_particle_size_dist(anim:fixed_dist(4.0), anim:fixed_dist(6.0))
  anim:activate()
end

function rose_fort_end(parent)
  game:show_game_over_window("Thanks for playing!  You have completed Act 1 of the Twin Expanse.  Please keep an eye out for future content and Act 2 of the campaign.")
end
