function on_area_load(parent)
  add_min_xp_coins()

  -- game:start_conversation("tervald", target)
end

MIN_COINS = 5000
MIN_XP = 800

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
