function on_area_load(parent)
  game:start_conversation("last_edge_intro", parent)
end

function trader_rest(parent)
  game:run_script_delayed("campaign", "fire_rest", 0.0)
end

function trader_quest_update(parent)
  game:set_quest_entry_state("the_aegis", "the_war", "Visible")
  game:set_world_map_location_visible("west_road", true)
  game:set_world_map_location_enabled("west_road", true)
end