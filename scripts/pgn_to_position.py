import re
import sys
from pathlib import Path

import chess


class DuckBoard(chess.Board):
    duck = 0

    def generate_legal_moves(self, from_mask=chess.BB_ALL, to_mask=chess.BB_ALL):
        return self.generate_pseudo_legal_moves(from_mask, to_mask & ~self.duck)

    def _attacked_for_king(self, *args):
        return False


pgn = Path(sys.argv[1]).read_text(encoding="utf-8-sig")
fen = re.search(r'\[FEN "([^"]+)"\]', pgn)[1]
board = DuckBoard(fen)
moves = []
for san, duck in re.findall(r'([A-Za-z0-9=+#-]+)@([a-h][1-8])', re.sub(r'\{[^}]*\}', '', pgn)):
    board.occupied |= board.duck
    move = board.parse_san(san)
    board.occupied &= ~board.duck
    board.push(move)
    board.duck = chess.BB_SQUARES[chess.parse_square(duck)]
    moves.append(f"{move.uci()},{chess.square_name(move.to_square)}{duck}")
print(f"position fen {fen} moves {' '.join(moves)}")
