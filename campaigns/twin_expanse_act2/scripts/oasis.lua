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