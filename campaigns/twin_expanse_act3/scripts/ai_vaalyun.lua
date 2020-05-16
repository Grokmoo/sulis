-- OnDamaged Hook
function on_damaged(parent, targets, hit)
    local target = targets:first()
	
	game:log(parent:name() .. " damaged by " .. target:name() .. ": "
	    .. hit:kind() .. " for " .. hit:total_damage() .. " damage.")
end

-- OnRoundElapsed Hook
function on_round_elapsed(parent)
  game:log(parent:name() .. " round elapsed.")
end