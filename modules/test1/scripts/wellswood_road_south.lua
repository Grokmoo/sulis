function on_talk_tervald(parent, target)
  --game:log("hello, world")
end

function on_player_enter_bridge(parent, target)
  game:log("Enter bridge")
  game:spawn_encounter_at(46, 72)
  game:spawn_encounter_at(16, 47)
end
