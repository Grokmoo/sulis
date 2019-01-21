function enable_naathfir(parent)
  game:set_world_map_location_visible("naathfir", true)
  game:set_world_map_location_enabled("naathfir", true)
end

function naathfir_road_thugs(parent)
  local thug = game:entity_with_id("naathfir_thug02")
  game:start_conversation("naathfir_road_thug", thug)
end

function set_road_thugs_hostile()
  local entities = game:entities_with_ids({"naathfir_thug01", "naathfir_thug02", "naathfir_thug03", "naathfir_thug04", "naathfir_thug05"})
  set_hostile(entities)
end

function set_hostile(entities)
  for i = 1, #entities do
    entities[i]:set_faction("Hostile")
  end
end

function trader_farewell(parent)
  game:set_quest_entry_state("dwarven_goods", "found", "Visible")
  local trader = game:entity_with_id("dwarf_trader01")
  
  if not trader:move_towards_point(21, 40) then
    game:warn("dwarf_trader01 unable to move")
  end
  
  game:run_script_delayed("naathfir", "trader_leave_finish", 2.0)
end

function trader_leave_finish(parent)
  local trader = game:entity_with_id("dwarf_trader01")
  trader:remove()
end

function guard_open_gate(parent)
  game:enable_prop_at(34, 7)
  game:toggle_prop_at(34, 7)
  game:set_quest_entry_state("the_aegis_gem", "mines", "Visible")
end

function mines_boss_enter(parent)
  game:cancel_blocking_anims()
  game:scroll_view(77, 54)

  local omonar = game:entity_with_id("mines_omonar")
  omonar:set_faction("Hostile")
  game:start_conversation("naathfir_mines_omonar", omonar)
  game:spawn_encounter_at(89, 42)
  game:set_quest_entry_state("the_aegis_gem", "lost", "Visible")
  game:enable_trigger_at(34, 20, "rose_fort_interior")
end