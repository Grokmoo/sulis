function on_area_load(parent)
  game:start_conversation("start", parent)
end

function on_boss_end(parent)
  game:show_game_over_window("Congratulations on completing the Endless Dungeon and thanks for playing!")
end