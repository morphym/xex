#!/usr/bin/env python3
"""
Targeted edge-case verification against python-chess.
Covers: castling (all four sides, blocked, through check, into check),
        en passant (normal, pinned pawn, last-rank edge),
        promotion (all four pieces, capture-promotion),
        and a random walk seeded to hit these cases frequently.
"""
import chess
import subprocess
import random
import sys

BINARY = "./target/debug/chess-moves"
failures = 0
checked  = 0


def our_moves(fen: str) -> set[str]:
    r = subprocess.run([BINARY, fen], capture_output=True, text=True)
    return set(r.stdout.split())


def check(label: str, fen: str) -> bool:
    global failures, checked
    board  = chess.Board(fen)
    ours   = our_moves(fen)
    theirs = {m.uci() for m in board.legal_moves}
    checked += 1
    if ours == theirs:
        print(f"  OK  {label}")
        return True
    failures += 1
    only_ours   = sorted(ours   - theirs)
    only_theirs = sorted(theirs - ours)
    print(f"  FAIL {label}")
    print(f"       fen : {fen}")
    if only_ours:   print(f"       extra in binary : {only_ours}")
    if only_theirs: print(f"       missing in binary: {only_theirs}")
    return False


# ── Castling ────────────────────────────────────────────────────────────────

print("── Castling ──")

# Both sides can castle both ways
check("white both castles available",
      "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1")
check("black both castles available",
      "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1")

# Castling rights stripped
check("no castling rights",
      "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w - - 0 1")
check("only white kingside",
      "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w K - 0 1")
check("only black queenside",
      "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b q - 0 1")

# Squares between king and rook occupied
check("kingside blocked by bishop",
      "r3k2r/8/8/8/8/8/8/R3KB1R w KQkq - 0 1")
check("queenside blocked by knight",
      "r3k2r/8/8/8/8/8/8/RN2K2R w KQkq - 0 1")

# King in check — cannot castle
check("king in check, no castling",
      "r3k2r/8/8/8/8/8/4r3/R3K2R w KQkq - 0 1")

# Castling through attacked square
check("white kingside through attacked f1",
      "r3k2r/8/8/8/8/8/8/R3K2r w KQ - 0 1")   # h1 rook attacks f1
check("white queenside through attacked d1",
      "r3k2r/8/8/8/8/8/8/r3K2R w KQ - 0 1")   # a1 rook attacks d1

# Castling into check (landing square attacked)
check("white kingside into check on g1",
      "r3k2r/8/8/8/8/8/6r1/R3K2R w KQ - 0 1")
check("white queenside into check on c1",
      "r3k2r/8/8/8/8/8/2r5/R3K2R w KQ - 0 1")

# Rook captured — rights should be gone (python-chess tracks via FEN)
check("castling right absent after rook capture (KQkq→Kkq encoded in fen)",
      "r3k2r/8/8/8/8/8/8/R3K2R w Kkq - 0 1")

# ── En passant ──────────────────────────────────────────────────────────────

print("\n── En passant ──")

# Standard en passant
check("white captures en passant left",
      "8/8/8/3pP3/8/8/8/4K2k w - d6 0 2")
check("white captures en passant right",
      "8/8/8/4Pp2/8/8/8/4K2k w - f6 0 2")
check("black captures en passant left",
      "4k2K/8/8/8/3pP3/8/8/8 b - e3 0 2")
check("black captures en passant right",
      "4k2K/8/8/8/4Pp2/8/8/8 b - e3 0 2")

# No en passant square set — should not generate ep move
check("en passant square absent",
      "8/8/8/3pP3/8/8/8/4K2k w - - 0 2")

# Pinned pawn cannot capture en passant (exposes king on rank)
check("ep illegal: pawn pinned on rank by rook",
      "8/8/8/K2pP2r/8/8/8/7k w - d6 0 2")

# En passant that reveals check on file
check("ep illegal: discovered check on file",
      "8/8/3k4/3pP3/8/8/8/3K4 w - d6 0 2")

# ── Promotion ───────────────────────────────────────────────────────────────

print("\n── Promotion ──")

# All four promotion pieces, straight push
check("white pawn promotes straight (4 choices)",
      "8/4P3/8/8/8/8/8/4K2k w - - 0 1")
check("black pawn promotes straight (4 choices)",
      "4k2K/8/8/8/8/8/4p3/8 b - - 0 1")

# Promotion via capture (two capture squares + straight = 12 moves)
check("white pawn promotes via capture left+right+straight",
      "3rr3/4P3/8/8/8/8/8/4K2k w - - 0 1")
check("black pawn promotes via capture",
      "4k2K/8/8/8/8/8/4p3/3RR3 b - - 0 1")

# Promotion on a-file and h-file (edge files, only one capture direction)
check("white pawn on a7 promotes",
      "8/P7/8/8/8/8/8/4K2k w - - 0 1")
check("white pawn on h7 promotes",
      "8/7P/8/8/8/8/8/4K2k w - - 0 1")

# ── Pinned pieces ────────────────────────────────────────────────────────────

print("\n── Pins & check evasions ──")

check("pinned pawn cannot push",
      "4k3/4r3/8/8/8/8/4P3/4K3 w - - 0 1")
check("pinned bishop cannot move off diagonal",
      "4k3/8/8/b7/8/8/8/B3K3 w - - 0 1")
check("only legal move is block or capture (single check)",
      "4k3/8/8/8/8/8/8/r3K3 w - - 0 1")
check("double check: only king move legal",
      "4k3/8/8/3b4/8/8/8/r3K3 w - - 0 1")

# ── Stalemate / checkmate positions ─────────────────────────────────────────

print("\n── Terminal positions ──")

check("stalemate (no legal moves, not in check)",
      "k7/8/1Q6/8/8/8/8/7K b - - 0 1")
check("checkmate (scholar's mate)",
      "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR b KQkq - 4 4")
check("checkmate back rank",
      "6k1/5ppp/8/8/8/8/8/6RK w - - 0 1")

# ── Random walk seeded for diversity ────────────────────────────────────────

print("\n── Random walk (500 games × 80 plies) ──")
ok_r = fail_r = 0
rng  = random.Random(42)
for seed in range(500):
    board = chess.Board()
    for _ in range(80):
        if board.is_game_over():
            break
        fen    = board.fen()
        ours   = our_moves(fen)
        theirs = {m.uci() for m in board.legal_moves}
        checked += 1
        if ours == theirs:
            ok_r += 1
        else:
            fail_r += 1
            failures += 1
            only_ours   = sorted(ours   - theirs)
            only_theirs = sorted(theirs - ours)
            print(f"  FAIL fen={fen}")
            if only_ours:   print(f"       extra  : {only_ours}")
            if only_theirs: print(f"       missing: {only_theirs}")
        board.push(rng.choice(list(board.legal_moves)))
    if (seed + 1) % 100 == 0:
        print(f"  {seed+1}/500 games — walk positions: {ok_r+fail_r}, failures: {fail_r}")

print(f"\n{'='*55}")
print(f"Total positions checked : {checked}")
print(f"Failures                : {failures}")
sys.exit(1 if failures else 0)
