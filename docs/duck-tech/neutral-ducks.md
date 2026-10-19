A lot of duck placements don't actually get in the opponents way. We call these 'neutral' ducks: Duck placements on squares that don't appear to restrict any of the opponents immediate moves. Their placement can still matter on later turns, but for now they are mostly just sitting there.

The following heuristics use this distinction to avoid spending too much time on neutral ducks.

**Neutral duck pruning** by Silverrzz\
https://github.com/Silverrzz/wakwak/pull/94\
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/move_picker.rs#L31-L54

The premise is as such: If a chess move has several neutral duck placements, we can assume that searching all of them is mostly wasted effort.

For each chess move, we therefore keep the neutral duck with the highest duck history score and prune the rest of the neutral ducks. Duck placements that could restrict the opponent are kept by this heuristic. So if a chess move has 20 neutral duck placements, we only need to search one of them!

In the main search, we only apply this pruning at non-PV nodes with a depth of 8 or less, and when neither alpha nor beta is a mate score. It is also used for noisy moves in quiescence search at non-PV nodes, with the same mate-score restriction.

**Neutral duck ordering** by Dan\

Give a move ordering penalty to 'neutral' ducks, prioritising duck placements that restrict the opponent's possible moves.

https://github.com/Silverrzz/wakwak/pull/100\
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/move_picker.rs#L353-L362
