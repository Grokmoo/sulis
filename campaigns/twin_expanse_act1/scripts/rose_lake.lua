function tour_guide(parent)
  game:cancel_blocking_anims()
  game:scroll_view(14, 113)
  game:start_conversation("rose_lake_tour_guide", game:player())
end

function council_secretary_suggestion(parent)
  game:set_quest_entry_state("seeing_the_council", "blocked", "Visible")
end