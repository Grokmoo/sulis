function priest_rest(parent)
  game:run_script_delayed("campaign", "fire_rest", 0.0)
end

function guide(parent)
  game:cancel_blocking_anims()
  game:scroll_view(22, 108)
  local guide = game:entity_with_id("oasis_guide")
  game:start_conversation("oasis_guide", guide)
end