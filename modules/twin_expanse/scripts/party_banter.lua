names = { "npc_aessa", "npc_grazi", "npc_jorzal" }

function on_enter_wellswood_road_north_cliffs(parent, target)
  party_member = get_party_member()
  if party_member == nil then return end
  
  game:say_line("I don't cherish the idea of heading into these hills - they are going to be swarming with goblins.", party_member)
end

function on_enter_wellswood_road_cave(parent, target)
  party_member = get_party_member()
  if party_member == nil then return end
  
  game:say_line("I doubt there is another way out of this cave, but I suppose it is worth a look...", party_member)
end

function on_complete_wellswood_road_cave(parent, target)
  game:spawn_encounter_at(49, 9)

  party_member = get_party_member()
  if party_member == nil then return end
  
  game:say_line("What the hell is that thing doing here?", party_member)
end

function on_enter_wellswood_road_north(parent, target)
  party_member = get_party_member()
  if party_member == nil then return end
  
  game:say_line("This forest just keeps going.  There must be a path through.", party_member)
end

function on_enter_wellswood_road_spiders(parent, target)
  party_member = get_party_member()
  if party_member == nil then return end
  
  game:say_line("I don't think the goblins come to this part of the forest.", party_member)
end

function get_party_member()
  targets = game:entities_with_ids(names)
  
  if #targets == 0 then
    return nil
  else
    -- random party member
    return targets[math.random(#targets)]
  end
end