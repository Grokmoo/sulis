function on_cave_load(player)
  game:run_script_delayed("campaign", "heal_party", 0.0)
  game:say_line("The magic in the air instantly heals your wounds and recovers your abilities.", game:player())
end