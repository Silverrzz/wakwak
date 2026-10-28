**Duck stability time management** by Tecci\
https://github.com/Silverrzz/wakwak/pull/104\
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/time.rs#L112-L129

A simple Time Management heuristic using the Duck. In standard Chess Engines, there is a
Time Management heuristic called Move Stability, which wakwak also has. Duck Stability is
very similar to Move Stability, in that we keep track of how often our Duck move changes.

If our Duck move is changing often, we should use more time to be more certain of our Duck placement
before committing to the move. That's the basic idea. On the other hand, if our Duck move hasn't changed
in a while, we should use less time because we are very certain of our Duck placement, so we shouldn't
waste more time on this move.
