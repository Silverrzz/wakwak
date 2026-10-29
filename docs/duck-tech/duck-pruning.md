The following heuristics overlap in concept to 'Late Move Pruning', a pruning heuristic which is universal in strong alpha/beta chess engines. LMP assumes that our move ordering is good, and that therefore moves ordered later in the move list are less likely to affect the outcome of the search, and can therefore be pruned.

In Duck Chess, each possible chess move comes with a set of possible duck placements. This introduces additional dimensions in which we can employ LMP-like pruning, as the heuristics below demonstrate.

**Duck Count Pruning (DCP)** by Dan\
https://github.com/Silverrzz/wakwak/pull/73

_"Only search X moves per duck"_.

We assume that after a certain number of moves containing a given duck placement are tried and failed, we can be reasonably confident that subsequent moves containing that duck placement will be just as bad, or worse. We therefore set a hard cap on the number of moves-per-duck-placement we search each node. That cap is calculated based on the search depth and the history score for the duck placement.

https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/search.rs#L518-L528

**Late Duck Pruning (LDP)** by Dan\
https://github.com/Silverrzz/wakwak/pull/45

_"Only search X ducks per move"_.

We assume that our ordering of the possible duck placements for a given move is good enough such that, if we have already searched a certain number of duck placements for a move, the remaining ducks are unlikely to be good enough to affect the outcome of the search. We therefore set a hard cap on the number of ducks we search per move, based on the search depth and the history score for the duck placement.

https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/search.rs#L530-L539

**Unique Duck Pruning (UDP)** by Dan\
https://github.com/Silverrzz/wakwak/pull/147

_"Only search X unique ducks per node"_.

We assume that duck placements that show up late in the move list are likely to be bad, and so we set a hard cap on the number of _unique_ duck placements we search per node, based on the search depth and the history score for the duck placement.

https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/search.rs#L541-L550
