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
end

function open_bridge(parent)
  game:disable_prop_at(34, 39)
  game:disable_prop_at(34, 41)
  
  game:toggle_prop_at(34, 39)
  game:toggle_prop_at(34, 41)
  guard02 = game:entity_with_id("guard02")
  if not guard02:move_towards_point(36, 41) then
    game:warn("guard02 unable to move")
  end
  
  game:set_world_map_location_enabled("rose_lake", true)
end