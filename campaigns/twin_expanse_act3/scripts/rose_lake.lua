function on_enter(parent)
  game:start_conversation("rose_lake_on_enter", parent)
end

function pull_lever(parent)
  game:set_quest_entry_state("naathfir_dwarves", "the_attack", "Visible")
  game:set_quest_state("naathfir_dwarves", "Complete")
  
  game:spawn_actor_at("councilor_berkeley", 110, 75, "Neutral", "rose_lake")
  
  game:spawn_actor_at("dwarf_raider01", 125, 68, "Neutral", "rose_lake")
  game:spawn_actor_at("dwarf_raider02", 125, 74, "Neutral", "rose_lake")
end