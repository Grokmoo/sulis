function tour_guide(parent)
  game:cancel_blocking_anims()
  game:scroll_view(14, 113)
  game:start_conversation("rose_lake_tour_guide", game:player())
end
