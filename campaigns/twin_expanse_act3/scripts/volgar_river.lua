function area_exit(parent)
  game:set_quest_entry_state("naathfir_dwarves", "rose_lake", "Visible")
  game:set_world_map_location_visible("rose_lake", true)
  game:set_world_map_location_enabled("rose_lake", true)
end