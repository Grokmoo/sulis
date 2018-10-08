function on_rest(parent)
  game:show_confirm("Rest now?", "Yes", "No", "campaign", "fire_rest")
end

function fire_rest(parent)
  game:fade_out_in()
  game:init_party_day()
end