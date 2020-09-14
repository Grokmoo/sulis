function on_enter(parent)
  local target = game:spawn_actor_at("dwarf_guard_captain", 77, 31)
  game:spawn_actor_at("dwarf_guard04", 88, 10)
  game:spawn_actor_at("dwarf_guard05", 79, 21)
  game:spawn_actor_at("dwarf_guard01", 95, 11)
  
  game:cancel_blocking_anims()
  game:scroll_view(82, 28)
  game:start_conversation("naathfir_on_enter", target)
end

function start_dwarves_quest(parent)
  game:set_quest_entry_state("naathfir_dwarves", "start", "Visible")
end