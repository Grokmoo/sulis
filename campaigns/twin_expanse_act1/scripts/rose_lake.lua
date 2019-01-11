function tour_guide(parent)
  game:cancel_blocking_anims()
  game:scroll_view(14, 113)
  game:start_conversation("rose_lake_tour_guide", game:player())
end

function council_secretary_suggestion(parent)
  game:set_quest_entry_state("seeing_the_council", "blocked", "Visible")
end

function weasel_end(parent)
  weasel = game:entity_with_id("rose_lake_weasel")
  coins = math.floor(weasel:get_num_flag("coins_to_take"))
  if coins > 0 then
    game:add_party_coins(coins)
  end
end

function weasel_debt_take_complete(parent)
  jevero = game:entity_with_id("rose_lake_q01")
  
  game:set_quest_entry_state("a_weasels_debt", "take_complete", "Visible")
  game:set_quest_state("a_weasels_debt", "Complete")
  
  percent_fee = (1 + math.floor(jevero:get_num_flag("negotiate"))) * 10
  
  coins = math.floor(jevero:get_num_flag("coins_to_take"))
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
  
  arzel = game:entity_with_id("arzel")
  arzel:set_faction("Hostile")
  
  game:start_conversation("arzel", parent)
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
  item = game:find_party_item("aegis_staff")
  game:remove_party_item(item)
  
  game:set_quest_entry_state("seeing_the_council", "complete", "Visible")
  game:set_quest_state("seeing_the_council", "Complete")
  
  game:set_world_map_location_visible("naathfir_road", true)
  game:set_world_map_location_enabled("naathfir_road", true)
end