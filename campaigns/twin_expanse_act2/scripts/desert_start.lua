function on_area_load(parent)
  add_min_xp_coins()
  
  game:set_quest_entry_state("the_aegis", "start", "Visible")
  game:start_conversation("intro", parent)
end

MIN_COINS = 50000
MIN_XP = 5000

function add_min_xp_coins()
  local player = game:player()
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
