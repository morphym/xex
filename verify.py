#!/usr/bin/env python3
"""
Random walk verification: compare move generation between our binary and python-chess.
Explores random game trees and reports any discrepancies.
"""
import chess
import subprocess
import random
import sys

BINARY = "./target/debug/chess-moves"


def our_moves(fen: str) -> set[str]:
    result = subprocess.run([BINARY, fen], capture_output=True, text=True)
    return set(result.stdout.split())


def their_moves(board: chess.Board) -> set[str]:
    return {m.uci() for m in board.legal_moves}


def check(board: chess.Board) -> bool:
    fen = board.fen()
    ours = our_moves(fen)
    theirs = their_moves(board)
    if ours == theirs:
        return True
    only_ours   = ours - theirs
    only_theirs = theirs - ours
    print(f"\nMISMATCH at {fen}")
    if only_ours:
        print(f"  only in binary  : {sorted(only_ours)}")
    if only_theirs:
        print(f"  only in python  : {sorted(only_theirs)}")
    return False


def random_walk(seed: int, depth: int) -> tuple[int, int]:
    """Play random moves up to `depth`, checking moves at each ply. Returns (ok, fail)."""
    rng = random.Random(seed)
    board = chess.Board()
    ok = fail = 0
    for _ in range(depth):
        if board.is_game_over():
            break
        if check(board):
            ok += 1
        else:
            fail += 1
        moves = list(board.legal_moves)
        board.push(rng.choice(moves))
    return ok, fail


def main():
    games   = int(sys.argv[1]) if len(sys.argv) > 1 else 200
    depth   = int(sys.argv[2]) if len(sys.argv) > 2 else 60
    total_ok = total_fail = 0

    print(f"Running {games} random games up to {depth} plies each…")
    for seed in range(games):
        ok, fail = random_walk(seed, depth)
        total_ok   += ok
        total_fail += fail
        if (seed + 1) % 20 == 0:
            print(f"  {seed+1}/{games} games — positions checked: {total_ok+total_fail}, failures: {total_fail}")

    print(f"\nDone. positions checked: {total_ok+total_fail}, failures: {total_fail}")
    sys.exit(1 if total_fail else 0)


if __name__ == "__main__":
    main()
