function on_enter(parent)
  game:start_conversation("rose_lake_on_enter", parent)
end

function pull_lever(parent)
  game:set_quest_entry_state("naathfir_dwarves", "the_attack", "Visible")
  game:set_quest_state("naathfir_dwarves", "Complete")
  
  local entity = game:spawn_actor_at("councilor_berkeley", 110, 75, "Neutral", "rose_lake")
  
  game:player():set_flag("rose_lake_berkeley_id", entity:id())
  
  game:spawn_actor_at("dwarf_raider01", 125, 68, "Neutral", "rose_lake")
  game:spawn_actor_at("dwarf_raider02", 125, 74, "Neutral", "rose_lake")
end

function on_area_load(parent)
  local berkeley_id = game:player():get_flag("rose_lake_berkeley_id")
  if berkeley_id == nil then return end

  local berkeley = game:entity_with_id(berkeley_id)
  if not berkeley:is_valid() then return end
  
  game:start_conversation("rose_lake_berkeley", berkeley)
end

function to_xandala(parent)
  game:set_quest_entry_state("the_aegis", "to_xandala", "Visible")
  game:transition_party_to(120, 120, "xandala")
  game:run_script_delayed("campaign", "heal_party", 0.0)
end