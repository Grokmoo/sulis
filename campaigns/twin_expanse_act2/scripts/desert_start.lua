function on_area_load(parent)
  add_min_xp_coins()
  setup_party()
  
  game:set_quest_entry_state("the_aegis", "start", "Visible")
  game:start_conversation("intro", parent)
end

MIN_COINS = 50000
MIN_XP = 5700

function add_min_xp_coins()
  local player = game:player()
  if player:has_flag("completed_twin_expanse_act1") then return end
  
  local stats = player:stats()
  
  local xp = stats.current_xp
  if xp < MIN_XP then
    player:add_xp(MIN_XP - xp)
  end
  
  local coins = game:party_coins()
  if coins < MIN_COINS then
    game:add_party_coins(MIN_COINS - coins)
  end
end

function setup_party()
  local player = game:player()
  if player:has_flag("completed_twin_expanse_act1") then return end
  
  local vaalyun = game:spawn_actor_at("npc_vaalyun", 10, 119)
  local cragnik = game:spawn_actor_at("npc_cragnik", 13, 114)
  local jhilsara = game:spawn_actor_at("npc_jhilsara", 6, 114)
  
  game:add_party_member(vaalyun:id())
  game:add_party_member(cragnik:id())
  game:add_party_member(jhilsara:id())
end