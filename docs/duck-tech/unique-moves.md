**Unique Move Pruning (UMP)** by Yoshie2000\
https://github.com/Silverrzz/wakwak/pull/154\
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/search.rs#L552-L561

This technique is similar to LMP in traditional chess engines. If we count the number of unique moves played, ignoring counts of the same move with different duck placements, we apply an LMP-like formula using this number to prune quiet moves.

**Unique move reductions** by Yoshie2000\
https://github.com/Silverrzz/wakwak/pull/155\
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/params.rs#L538-L544

Traditional chess engines apply a base LMR reduction using `log(depth)` and `log(moves_tried)`. Since in duck chess, `moves_tried` is much larger, and there are multiple duck placements per move, we re-use the `unique_moves` counter from UMP to calculate this reduction instead.
