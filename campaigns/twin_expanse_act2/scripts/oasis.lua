function priest_rest(parent)
  game:run_script_delayed("campaign", "fire_rest", 0.0)
end

function guide(parent)
  game:cancel_blocking_anims()
  game:scroll_view(22, 108)
  local guide = game:entity_with_id("oasis_guide")
  game:start_conversation("oasis_guide", guide)
end

function working_for_the_boss_a(parent)
  game:set_quest_entry_state("working_for_the_boss", "start", "Visible")
  game:set_world_map_location_visible("drake_nest", true)
  game:set_world_map_location_enabled("oasis", true)
end

function fire_drakes_cleared(parent)
  game:set_quest_entry_state("working_for_the_boss", "drakes_cleared", "Visible")
  game:player():set_flag("drakes_cleared", "true")
end

function working_for_the_boss_b(parent)
  game:set_world_map_location_visible("dracon_camp", true)
  game:set_quest_entry_state("working_for_the_boss", "dracon_camp", "Visible")
  game:player():set_flag("dracon_assigned", "true")
end

function herbalist_quest_start(parent)
  game:set_quest_entry_state("attacked_in_the_dark", "start", "Visible")
  game:set_world_map_location_visible("desert_canyon", true)
  game:set_world_map_location_enabled("oasis", true)
end

function herbalist_supplies_find(parent)
  game:set_quest_entry_state("attacked_in_the_dark", "found", "Visible")
  game:spawn_actor_at("sand_elemental", 35, 116)
end

function herbalist_quest_end(parent, target)
  target:set_flag("supplies_found", "true")
  game:set_quest_entry_state("attacked_in_the_dark", "returned", "Visible")
  game:set_quest_state("attacked_in_the_dark", "Complete")
  game:add_party_coins(8000)
  
  local item = game:find_party_item("merchant_supplies")
  game:remove_party_item(item)
end

function missing_mercs_start(parent)
  game:set_quest_entry_state("missing_mercs", "start", "Visible")
  game:set_world_map_location_visible("desert_canyon", true)
  game:set_world_map_location_enabled("oasis", true)
end

function mercs_found(parent)
  game:set_quest_entry_state("missing_mercs", "found", "Visible")
  game:block_ui(4.0)
  game:fade_out_in()
  game:cancel_blocking_anims()
  game:run_script_delayed("oasis", "remove_mercs", 2.0)
  game:player():set_flag("mercs_found")
end

function remove_mercs(parent)
  local target = game:entity_with_id("oasis_merc01")
  target:remove()
  local target = game:entity_with_id("oasis_merc02")
  target:remove()
  local target = game:entity_with_id("oasis_merc03")
  target:remove()
  local target = game:entity_with_id("oasis_merc04")
  target:remove()
end

function missing_mercs_complete(parent, target)
  game:player():clear_flag("mercs_found")
  target:set_flag("quest_complete")
  game:add_party_coins(6000)
  
  game:set_quest_entry_state("missing_mercs", "complete", "Visible")
  game:set_quest_state("missing_mercs", "Complete")
end

function dracon_imperator_show_library(parent, target)
  game:set_quest_entry_state("the_aegis", "the_library", "Visible")
  game:set_quest_entry_state("working_for_the_boss", "dracon_allies", "Visible")
  
  game:set_world_map_location_visible("ancient_library", true)
end

function dracon_imperator_show_mesa(parent, target)
  game:set_quest_entry_state("the_aegis", "razethar_the_sage", "Visible")
  
  game:set_world_map_location_visible("lonely_mesa", true)
  game:set_world_map_location_visible("southern_desert", true)
end

function on_exit_lonely_mesa(parent)
  game:set_world_map_location_enabled("lonely_mesa", true)
end

function flameling_ambush_1(parent)
  game:spawn_encounter_at(16, 51)
end

function flameling_ambush_2(parent)
  game:spawn_encounter_at(28, 47)
end

function flameling_ambush_3(parent)
  game:spawn_encounter_at(49, 77)
end

function razethar_spoke(parent)
  game:set_quest_entry_state("the_aegis", "spoke_to_razethar", "Visible")
  game:player():set_flag("spoke_to_razethar")
end

function remove_aegis_book(parent)
  local item = game:find_party_item("history_of_the_aegis")
  game:remove_party_item(item)
end

function open_blazing_road(parent)
  game:player():clear_flag("spoke_to_razethar")
  game:enable_prop_at(96, 70)
  game:set_world_map_location_visible("blazing_road_west", true)
  game:set_world_map_location_enabled("blazing_road_west", true)
end

function exit_blazing_road_west(parent)
  game:set_world_map_location_visible("blazing_road_east", true)
  game:set_world_map_location_enabled("blazing_road_east", true)
end

function on_enter_blazing_road_exit(parent)
  game:block_ui(4.0)
  game:fade_out_in()
  
  game:cancel_blocking_anims()
  
  game:run_script_delayed("oasis", "blazing_road_exit_spawn", 2.0)
end

function blazing_road_exit_spawn(parent)
  game:spawn_encounter_at(34, 9)
  game:scroll_view(39, 18)
  
  local target = game:entity_with_id("blazing_road_boss")
  game:start_conversation("blazing_road_boss", target)
  game:check_ai_activation(parent)
  
  local dest = { {x=39, y=22}, {x=42, y=23}, {x=36, y=23}, {x=39, y=25}}
  local party = game:party()
  for i = 1, #party do
    local member = party[i]
	member:teleport_to(dest[i])
  end
end

function on_blazing_road_exit_cleared(parent)
  game:block_ui(4.0)
  game:fade_out_in()
  game:scroll_view(39, 5)
  game:cancel_blocking_anims()
  game:run_script_delayed("oasis", "campaign_end", 3.0)
end

function campaign_end(parent)
  remove_items({"merchant_supplies", "history_of_the_aegis"})

  game:show_game_over_window("Congratulations, you have completed Act 2 of the Twin Expanse.  Please keep an eye on the project in the coming months for the release of the third and final act.  Thanks for playing!")

  --local export = game:create_module_export("twin_expanse_act2")
  --export:set_include_stash(true)
  --export:set_flag("completed_twin_expanse_act2")

  --local player = game:player()
  --local party = game:party()
  --for i = 1, #party do
  --    local member = party[i]
  --    if not member:has_flag("__is_summoned_party_member") and player:id() ~= member:id() then
  --        export:add_to_party(party[i])
  --    end
  --end
  --export:activate()
end

function remove_items(ids)
  for i = 1, #ids do
    item = game:find_party_item(ids[i])
	if item:is_valid() then
	  game:remove_party_item(item)
	end
  end
end