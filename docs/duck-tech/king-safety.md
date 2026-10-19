**Duck or Die Pruning** by Silverrzz\
https://github.com/Silverrzz/wakwak/pull/49\
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/search/search.rs#L618-L627

Duck or Die Pruning is just an overdramatic way of reintroducing the concept of 'check'. In Duck Chess, there is no 'check' or 'checkmate', so it is fully legal to make a Chess move that puts/leaves the king in danger and then place the duck somewhere that keeps the king in danger. But just because it's legal doesn't mean we should! So Duck or Die Pruning scores duck placements that leave our king under attack as immediate losses, without searching them further, unless the move produces a draw or has already captured the opponent's king.

**Duck shield evaluation** by Paul\
https://github.com/Silverrzz/wakwak/pull/92\
https://github.com/Silverrzz/wakwak/blob/736fd66018f03a4588e6052c4c251e327b56fdbb/src/eval/duck_shield.rs#L6-L68
