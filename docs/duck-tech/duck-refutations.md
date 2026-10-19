**Duck refutations** by Silverrzz\
https://github.com/Silverrzz/wakwak/pull/50\
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/search.rs#L485-L491

Duck refutations is an extremely potent duck-pruning method, that is a totally no-brainer once understood. After we make a chess move and a duck move, if our opponent immediately 'refutes' the chess move we made (By raising the opponents alpha) then we should prune any ducks for this chess move that don't block the opponents refutation.
