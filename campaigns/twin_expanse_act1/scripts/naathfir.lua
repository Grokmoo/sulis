function enable_naathfir(parent)
  game:set_world_map_location_visible("naathfir", true)
  game:set_world_map_location_enabled("naathfir", true)
end

function naathfir_road_thugs(parent)
  thug = game:entity_with_id("naathfir_thug02")
  game:start_conversation("naathfir_road_thug", thug)
end

function set_road_thugs_hostile()
  entities = game:entities_with_ids({"naathfir_thug01", "naathfir_thug02", "naathfir_thug03", "naathfir_thug04", "naathfir_thug05"})
  set_hostile(entities)
end

function set_hostile(entities)
  for i = 1, #entities do
    entities[i]:set_faction("Hostile")
  end
end

function trader_farewell(parent)
  game:set_quest_entry_state("dwarven_goods", "found", "Visible")
  trader = game:entity_with_id("dwarf_trader01")
  
  if not trader:move_towards_point(21, 40) then
    game:warn("dwarf_trader01 unable to move")
  end
  
  game:run_script_delayed("naathfir", "trader_leave_finish", 2.0)
end

function trader_leave_finish(parent)
  trader = game:entity_with_id("dwarf_trader01")
  trader:remove()
end