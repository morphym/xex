use chess::Board;
use std::io::{self, BufRead};

/// Reads FEN strings (one per line) from stdin or as a command-line argument,
/// and writes legal moves (one per line, UCI format) to stdout.
///
/// Usage:
///   chess-moves "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
///   echo "<fen>" | chess-moves
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            emit(line.expect("io error").trim());
        }
    } else {
        emit(&args.join(" "));
    }
}

fn emit(fen: &str) {
    if fen.is_empty() { return; }
    match Board::from_fen(fen) {
        Some(board) => {
            for mv in board.legal_moves() {
                println!("{mv}");
            }
        }
        None => eprintln!("invalid fen: {fen}"),
    }
}
