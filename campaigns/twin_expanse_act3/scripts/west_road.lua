function road_blocked(parent)
  game:cancel_blocking_anims()
  game:scroll_view(116, 96)
  game:start_conversation("west_road_blocked", parent)
end