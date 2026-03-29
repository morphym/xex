# xex

Minimal chess move generation library and binary.
The board is a geometric coordinate system: file `0–7` (a–h), rank `0–7` (1–8).
Rules are fully enforced: castling, en passant, promotion, check, checkmate, stalemate.

---

## Verification

Move generation is verified against (`pip install chess`).

**General random walk** — `verify.py`
200 random games × 60 plies = **11,989 positions**, 0 failures.

```sh
python3 verify.py [games] [plies]
```

Test run

```sh
Running 200 random games up to 60 plies each…
  20/200 games — positions checked: 1200, failures: 0
  40/200 games — positions checked: 2400, failures: 0
  60/200 games — positions checked: 3600, failures: 0
  80/200 games — positions checked: 4800, failures: 0
  100/200 games — positions checked: 5989, failures: 0
  120/200 games — positions checked: 7189, failures: 0
  140/200 games — positions checked: 8389, failures: 0
  160/200 games — positions checked: 9589, failures: 0
  180/200 games — positions checked: 10789, failures: 0
  200/200 games — positions checked: 11989, failures: 0

Done. positions checked: 11989, failures: 0
```

```






pawit@Pawits-MacBook-Air xex % python3 verify_special.py 
── Castling ──
  OK  white both castles available
  OK  black both castles available
  OK  no castling rights
  OK  only white kingside
  OK  only black queenside
  OK  kingside blocked by bishop
  OK  queenside blocked by knight
  OK  king in check, no castling
  OK  white kingside through attacked f1
  OK  white queenside through attacked d1
  OK  white kingside into check on g1
  OK  white queenside into check on c1
  OK  castling right absent after rook capture (KQkq→Kkq encoded in fen)

── En passant ──
  OK  white captures en passant left
  OK  white captures en passant right
  OK  black captures en passant left
  OK  black captures en passant right
  OK  en passant square absent
  OK  ep illegal: pawn pinned on rank by rook
  OK  ep illegal: discovered check on file

── Promotion ──
  OK  white pawn promotes straight (4 choices)
  OK  black pawn promotes straight (4 choices)
  OK  white pawn promotes via capture left+right+straight
  OK  black pawn promotes via capture
  OK  white pawn on a7 promotes
  OK  white pawn on h7 promotes

── Pins & check evasions ──
  OK  pinned pawn cannot push
  OK  pinned bishop cannot move off diagonal
  OK  only legal move is block or capture (single check)
  OK  double check: only king move legal

── Terminal positions ──
  OK  stalemate (no legal moves, not in check)
  OK  checkmate (scholar's mate)
  OK  checkmate back rank

── Random walk (500 games × 80 plies) ──
  100/500 games — walk positions: 7917, failures: 0
  200/500 games — walk positions: 15868, failures: 0
  300/500 games — walk positions: 23819, failures: 0
  400/500 games — walk positions: 31782, failures: 0
  500/500 games — walk positions: 39752, failures: 0

=======================================================
Total positions checked : 39785
Failures                : 0```

**Targeted edge cases** — `verify_special.py`
Explicit positions for every special rule, plus a 500-game × 80-ply random walk = **39,785 positions**, 0 failures.

```sh
python3 verify_special.py
```

Edge cases covered:
- Castling — both sides, blocked squares, king in check, through attacked square, into check, stripped rights
- En passant — both colors, absent EP square, pawn pinned on rank, discovered check on file
- Promotion — all four pieces, straight push, capture, edge files (a/h)
- Pins and check evasions — pinned pawn, pinned bishop, single check, double check
- Terminal positions — stalemate, checkmate

---

## Binary — `chess-moves`

Reads a FEN string, writes every legal move to stdout (one per line, UCI format).

### Build

```sh
cargo build --release
# binary at: target/release/chess-moves
```

### Usage

**Argument:**
```sh
chess-moves "<fen>"
```

**Stdin (one FEN per line):**
```sh
echo "<fen>" | chess-moves
```

**Parallel (GNU parallel / xargs):**
```sh
cat fens.txt | parallel chess-moves {}
cat fens.txt | xargs -P8 -I{} chess-moves {}
```

### Example

```sh
$ chess-moves "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
a2a3
a2a4
b1a3
b1c3
...           # 20 moves total
```

Move format: `<from><to>[promotion]`
Promotion piece: `q` queen, `r` rook, `b` bishop, `n` knight
Example promotion: `e7e8q`

---

## Library — `xex`

### Add as dependency

From a local path:
```toml
[dependencies]
chess = { path = "../chess" }
```

### Types

| Type | Description |
|------|-------------|
| `Sq` | Square: `file` and `rank` (both `u8`, `0–7`) |
| `Move` | `from: Sq`, `to: Sq`, `promotion: Option<Piece>` |
| `Board` | Position: squares, side to move, castling rights, en passant |
| `Color` | `White` \| `Black` |
| `Piece` | `Pawn` \| `Knight` \| `Bishop` \| `Rook` \| `Queen` \| `King` |

### Board construction

```rust
// Starting position
let board = Board::starting_position();

// From FEN
let board = Board::from_fen(
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
).unwrap();
```

### Move generation

```rust
let moves: Vec<Move> = board.legal_moves();
```

### Applying a move

`apply_move` is non-destructive — it returns a new `Board`.

```rust
let next: Board = board.apply_move(moves[0]);
```

### Square construction

```rust
let sq = Sq::new(4, 1);                    // e2
let sq = Sq::from_algebraic("e2").unwrap();
println!("{sq}");                           // "e2"
```

### Move display (UCI)

```rust
println!("{mv}");   // e.g. "e2e4", "e7e8q"
```

### Game state queries

```rust
board.is_in_check(Color::White)  // → bool
board.is_checkmate()             // current side to move
board.is_stalemate()             // current side to move
```

### Full example

```rust
use chess::{Board, Color};

fn main() {
    let mut board = Board::starting_position();
    loop {
        let moves = board.legal_moves();
        if moves.is_empty() {
            if board.is_checkmate() { println!("checkmate"); }
            else                    { println!("stalemate"); }
            break;
        }
        println!("turn={:?}  moves={}", board.turn, moves.len());
        board = board.apply_move(moves[0]);
    }
}
```

### Board fields

```rust
board.turn          // Color — side to move
board.castling      // [bool; 4] — [WK, WQ, BK, BQ]
board.en_passant    // Option<Sq> — en passant target square
board.get(sq)       // Option<(Color, Piece)>
```
