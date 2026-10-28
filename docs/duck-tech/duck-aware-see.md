**Duck-aware SEE move ordering** by Tecci\
https://github.com/Silverrzz/wakwak/pull/142\
https://github.com/Silverrzz/wakwak/blob/40a3bca77f39a1b539d6dcb6f205fc202a356a10/src/board/see.rs#L28-L136

Normal **Static Exchange Evaluation** (**SEE**) doesn't work well in Duck Chess,
because it fails to consider the Duck's movement. Therefore, we need to make a few
adjustments for it to work in Duck Chess.

Firstly, consider that in exchange evaluation, the Duck can **only** block sliders.
This is because pawns and kings can only capture from one square away, and
knights can always jump over the Duck directly to the capture square.

In the modified algorithm, we keep track of a **Bitboard** that contains the Duck's current square.
The Duck is taken into account when calculating slider rays to select our next piece.
After capturing, when selecting the opponent's next piece to capture, we either:

- Put the Duck between the slider and the capture square to block its ray.
- Or we remove the Duck from the bitboard.

Using a bitboard instead of a square makes it easier to model the fact that we can always
choose the best square to put the Duck. Otherwise, we would need to put the Duck on some
arbitrary square where it could still inadvertently interfere with the exchange evaluation.

Lastly, another difference from the normal SEE algorithm is that the King *can* capture in
certain situations, because the Duck can block an attack on the King. Therefore, instead of
simply resigning ourselves to the fact that we lost the exchange if our last piece to take was
our King, we can just check whether the King can still be captured before making that decision.
