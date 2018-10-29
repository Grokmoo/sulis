function learn_bridge_closed(parent)
  game:set_quest_entry_state("entering_rose_lake", "start", "Visible")
end

function learn_bridge_pass(parent)
  game:set_quest_entry_state("entering_rose_lake", "learn_pass", "Visible")
  game:player():set_flag("rose_lake_bridge_pass")
end

function mayor_fenk_ask_pass(parent)
  game:set_quest_entry_state("entering_rose_lake", "mayor_fenk_ask_pass", "Visible")
end

function exit_to_rose_lake(parent)
  game:set_world_map_location_enabled("rose_lake", true)
end