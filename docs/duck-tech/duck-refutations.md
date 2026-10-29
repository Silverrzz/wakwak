**Duck refutations** by Silverrzz\
https://github.com/Silverrzz/wakwak/pull/50
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/search.rs#L485-L491

Duck refutations is an extremely potent duck-pruning method, that is a totally no-brainer once understood. After we make a chess move and a duck move, if our opponent immediately 'refutes' the chess move we made (By raising the opponent's alpha) then we should prune any ducks for this chess move that don't block the opponent's refutation.

Duck refutations on their own may miss draws by repetition, if a duck move leading to a repetition is refuted by a previously searched duck move that doesn't lead to repetition. For example: <https://cope-chess.live/tournaments/154?game_id=330719>. To prevent this, we use upcoming repetition detection, so alpha gets raised to a draw score, which will be correct even if the falsely refuted move gets pruned. The upcoming repetition detection is a duckchess adaptation of the cuckoo hashing based detection presented in <https://web.archive.org/web/20201107002606/https://marcelk.net/2013-04-06/paper/upcoming-rep-v2.pdf>, and was added in <https://github.com/Silverrzz/wakwak/pull/218>.
