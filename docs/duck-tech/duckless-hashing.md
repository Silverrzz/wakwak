**Duckless hashing** by Silverrzz\
https://github.com/Silverrzz/wakwak/pull/66
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/search.rs#L277-L299

Duckless hashing allows us to store a chess position in the Transposition Table without including the duck's square in the hash. If we have already found a good move in a position, and we reach the same position with the duck somewhere else, that move may still be just as good.

The important part is that we check whether the entire move (The chess move and the duck placement) is still legal. If it is, playing it gives us the same resulting position, as the duck is moved to the same square regardless of where it started. We can therefore reuse what we already know about that move, instead of searching it again.

This doesn't mean that both starting positions have the same score. The different duck placement might allow an even better move, or block another move that was previously available. We therefore only store a lower bound: We know we have a move worth at least this much, but there may be something better.

If the stored result was searched deeply enough and its score is at least beta, we can use it for a cutoff at a non-PV node. Otherwise, we can use the stored move for move ordering when we don't already have a TT move. Mate scores are excluded from duckless hashing.
