function goblin_raids_start(parent)
  game:set_quest_entry_state("leader_of_beasts", "start", "Visible")
end

function goblin_raids_leads(parent)
  game:set_quest_entry_state("leader_of_beasts", "leads", "Visible")
end

function cragnik_join(parent)
  game:add_party_member("npc_cragnik")
end

function enter_square(parent)
  game:cancel_blocking_anims()
  game:scroll_view(98, 47)
  game:start_conversation("wellswood_enter_square", parent)
end

function enter_docks(parent)
  game:cancel_blocking_anims()
  game:scroll_view(61, 107)
  game:start_conversation("wellswood_enter_docks", parent)
end

function view_docks_thug(parent)
  game:set_quest_entry_state("the_thug", "docks_view", "Visible")
  
  coins = game:party_coins()
  if coins > 500 then
    parent:set_flag("has_50_coins")
  end
  
  if coins > 1000 then
    parent:set_flag("has_100_coins")
  end
end

function docks_thugs_attack(parent)
  -- TODO set thugs to hostile disposition
end

function docks_thugs_leave(parent)
  thug01 = game:entity_with_id("thug01")
  thug02 = game:entity_with_id("thug02")
  thug03 = game:entity_with_id("thug03")
  
  if not thug01:move_towards_point(75, 77) then
    game:log("thug01 unable to move")
  end
  
  if not thug02:move_towards_point(77, 77) then
    game:log("thug02 unable to move")
  end
  
  if not thug03:move_towards_point(79, 77) then
    game:log("thug02 unable to move")
  end
  
  game:run_script_delayed("wellswood", "docks_thugs_leave_finish", 2.0)
end

function docks_thugs_leave_finish(parent)
  target = game:entity_with_id("thug01")
  target:remove()
  
  target = game:entity_with_id("thug02")
  target:remove()
  
  target = game:entity_with_id("thug03")
  target:remove()
end

function docks_thugs_cleared(parent)
  -- TODO start dock worker convo
end

function docks_thugs_pay50(parent)
  game:add_party_coins(-500)
end

function docks_thugs_pay100(parent)
  game:add_party_coins(-1000)
end

function priest_rest(parent)
  game:fade_out_in()
  game:init_party_day()
end