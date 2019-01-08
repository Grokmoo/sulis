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