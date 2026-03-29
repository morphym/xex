# chess

Minimal chess move generation library and binary.
The board is a geometric coordinate system: file `0–7` (a–h), rank `0–7` (1–8).
Rules are fully enforced: castling, en passant, promotion, check, checkmate, stalemate.

---

## Verification

Move generation is verified against [python-chess](https://python-chess.readthedocs.io/) (`pip install chess`).

**General random walk** — `verify.py`
200 random games × 60 plies = **11,989 positions**, 0 failures.

```sh
python3 verify.py [games] [plies]
```

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

## Library — `chess`

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
