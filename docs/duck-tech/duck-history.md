**Duck history** by Dan\
https://github.com/Silverrzz/wakwak/pull/42

Extending the 'history heuristic' common in standard chess engines, we keep track of how duck placements have performed during search. On a beta cutoff, we give a bonus to the best move's duck placement and a penalty to all the other duck placements tried in that node. This history score influences the order in which we try moves in subsequent searches.

The indexing is simply `[stm][duck_square]`.

https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/history/duck.rs#L8-L39

**Quiet duck history** by Tecci\
https://github.com/Silverrzz/wakwak/pull/158
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/history/quiet_duck.rs#L8-L41

In standard chess engines, there is a Quiet History typically indexed by `[stm][src][dest]` among other things.
In wakwak, there is a Quiet Duck History alongside the normal Quiet History, indexed by `[stm][piece][dest][duck]`.
We index by `[piece]` instead of `[src]` in order to reduce the size of the history table.
