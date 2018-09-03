function disable_go_too_far()
  game:disable_trigger_at(3, 27)
end

function on_add_aessa(parent, target)
  game:add_party_member("npc_aessa")
  disable_go_too_far()
end

function on_add_jorzal(parent, target)
  game:add_party_member("npc_jorzal")
  disable_go_too_far()
end

function on_add_grazi(parent, target)
  game:add_party_member("npc_grazi")
  disable_go_too_far()
end

function on_player_enter_bridge(parent, target)
  game:cancel_blocking_anims()
  game:spawn_encounter_at(17, 36)
  game:enable_trigger_at(35, 65)
end

function on_player_go_too_far(parent, target)
  game:say_line("I should check back with the others before going any further.", parent)
  game:cancel_blocking_anims()
end

function on_player_return(parent, target)
  game:spawn_encounter_at(49, 81)
end

function on_area_load(parent)
  target = game:entity_with_id("npc_tervald")
  
  add_min_xp_coins()
  
  base_class = game:player():base_class()
  if base_class ~= "fighter" then
    target:set_flag("jorzal_valid_pick")
  end
  
  if base_class ~= "rogue" then
    target:set_flag("grazi_valid_pick")
  end
  
  if base_class ~= "mage" then
    target:set_flag("aessa_valid_pick")
  end
  
  game:start_conversation("tervald", target)
end

MIN_COINS = 0
MIN_XP = 0

function add_min_xp_coins()
  player = game:player()
  stats = player:stats()
  
  xp = stats.current_xp
  if xp < MIN_XP then
    player:add_xp(MIN_XP - xp)
  end
  
  coins = game:party_coins()
  if coins < MIN_COINS then
    game:add_party_coins(MIN_COINS - coins)
  end
end

function on_ambush_cleared(parent)
  target = game:entity_with_id("npc_tervald")
  target:set_flag("ambush_cleared")
end
