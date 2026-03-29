use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn flip(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// Square as geometric coordinate: file 0=a..7=h, rank 0=1..7=8
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Sq {
    pub file: u8,
    pub rank: u8,
}

impl Sq {
    pub fn new(file: u8, rank: u8) -> Self {
        Sq { file, rank }
    }

    pub fn from_algebraic(s: &str) -> Option<Self> {
        let b = s.as_bytes();
        if b.len() < 2 { return None; }
        let f = b[0].wrapping_sub(b'a');
        let r = b[1].wrapping_sub(b'1');
        if f > 7 || r > 7 { return None; }
        Some(Sq { file: f, rank: r })
    }

    fn offset(self, df: i8, dr: i8) -> Option<Self> {
        let f = self.file as i8 + df;
        let r = self.rank as i8 + dr;
        if (0..8).contains(&f) && (0..8).contains(&r) {
            Some(Sq { file: f as u8, rank: r as u8 })
        } else {
            None
        }
    }
}

impl fmt::Display for Sq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", (b'a' + self.file) as char, (b'1' + self.rank) as char)
    }
}

/// A move: from square, to square, optional promotion piece
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    pub from: Sq,
    pub to: Sq,
    pub promotion: Option<Piece>,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(p) = self.promotion {
            write!(f, "{}", match p {
                Piece::Queen  => 'q',
                Piece::Rook   => 'r',
                Piece::Bishop => 'b',
                Piece::Knight => 'n',
                _             => return Ok(()),
            })?;
        }
        Ok(())
    }
}

/// Board position
#[derive(Clone, Debug)]
pub struct Board {
    /// squares[rank][file]
    squares: [[Option<(Color, Piece)>; 8]; 8],
    pub turn: Color,
    /// [white kingside, white queenside, black kingside, black queenside]
    pub castling: [bool; 4],
    pub en_passant: Option<Sq>,
}

impl Board {
    pub fn starting_position() -> Self {
        let mut b = Board {
            squares: [[None; 8]; 8],
            turn: Color::White,
            castling: [true; 4],
            en_passant: None,
        };
        let back = [Piece::Rook, Piece::Knight, Piece::Bishop, Piece::Queen,
                    Piece::King, Piece::Bishop, Piece::Knight, Piece::Rook];
        for (f, &p) in back.iter().enumerate() {
            b.squares[0][f] = Some((Color::White, p));
            b.squares[7][f] = Some((Color::Black, p));
        }
        for f in 0..8 {
            b.squares[1][f] = Some((Color::White, Piece::Pawn));
            b.squares[6][f] = Some((Color::Black, Piece::Pawn));
        }
        b
    }

    pub fn from_fen(fen: &str) -> Option<Self> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        let rows: Vec<&str> = parts.first()?.split('/').collect();
        if rows.len() != 8 { return None; }

        let mut squares = [[None; 8]; 8];
        for (ri, row) in rows.iter().enumerate() {
            let rank = 7 - ri;
            let mut file = 0usize;
            for c in row.chars() {
                if let Some(n) = c.to_digit(10) {
                    file += n as usize;
                } else {
                    if file >= 8 { return None; }
                    let color = if c.is_uppercase() { Color::White } else { Color::Black };
                    let piece = match c.to_ascii_lowercase() {
                        'p' => Piece::Pawn,
                        'n' => Piece::Knight,
                        'b' => Piece::Bishop,
                        'r' => Piece::Rook,
                        'q' => Piece::Queen,
                        'k' => Piece::King,
                        _   => return None,
                    };
                    squares[rank][file] = Some((color, piece));
                    file += 1;
                }
            }
        }

        let turn = match parts.get(1).copied() {
            Some("b") => Color::Black,
            _         => Color::White,
        };

        let cs = parts.get(2).copied().unwrap_or("KQkq");
        let castling = [cs.contains('K'), cs.contains('Q'), cs.contains('k'), cs.contains('q')];

        let en_passant = match parts.get(3).copied() {
            Some("-") | None => None,
            Some(s)          => Sq::from_algebraic(s),
        };

        Some(Board { squares, turn, castling, en_passant })
    }

    pub fn get(&self, sq: Sq) -> Option<(Color, Piece)> {
        self.squares[sq.rank as usize][sq.file as usize]
    }

    fn set(&mut self, sq: Sq, val: Option<(Color, Piece)>) {
        self.squares[sq.rank as usize][sq.file as usize] = val;
    }

    /// All legal moves for the side to move
    pub fn legal_moves(&self) -> Vec<Move> {
        self.pseudo_moves()
            .into_iter()
            .filter(|&m| !self.leaves_in_check(m))
            .collect()
    }

    /// Apply a move, returning the new board state
    pub fn apply_move(&self, mv: Move) -> Board {
        let mut b = self.clone();
        let mover = b.get(mv.from);
        b.en_passant = None;

        match mover {
            Some((color, Piece::Pawn)) => {
                let dir: i8 = if color == Color::White { 1 } else { -1 };
                // En passant capture: diagonal move onto empty square
                if mv.from.file != mv.to.file && b.get(mv.to).is_none() {
                    b.set(Sq::new(mv.to.file, (mv.to.rank as i8 - dir) as u8), None);
                }
                // Double push: record en passant target square
                if (mv.to.rank as i8 - mv.from.rank as i8).abs() == 2 {
                    b.en_passant = Some(Sq::new(mv.from.file, (mv.from.rank as i8 + dir) as u8));
                }
                let placed = mv.promotion.map(|p| (color, p)).or(mover);
                b.set(mv.from, None);
                b.set(mv.to, placed);
            }
            Some((color, Piece::King)) => {
                let r = if color == Color::White { 0u8 } else { 7u8 };
                if mv.from == Sq::new(4, r) {
                    if mv.to == Sq::new(6, r) {
                        b.set(Sq::new(7, r), None);
                        b.set(Sq::new(5, r), Some((color, Piece::Rook)));
                    } else if mv.to == Sq::new(2, r) {
                        b.set(Sq::new(0, r), None);
                        b.set(Sq::new(3, r), Some((color, Piece::Rook)));
                    }
                }
                if color == Color::White { b.castling[0] = false; b.castling[1] = false; }
                else                     { b.castling[2] = false; b.castling[3] = false; }
                b.set(mv.from, None);
                b.set(mv.to, mover);
            }
            Some((color, Piece::Rook)) => {
                let r = if color == Color::White { 0u8 } else { 7u8 };
                let (ks, qs) = if color == Color::White { (0, 1) } else { (2, 3) };
                if mv.from == Sq::new(7, r) { b.castling[ks] = false; }
                if mv.from == Sq::new(0, r) { b.castling[qs] = false; }
                b.set(mv.from, None);
                b.set(mv.to, mover);
            }
            _ => {
                b.set(mv.from, None);
                b.set(mv.to, mover);
            }
        }

        // Revoke castling if a rook is captured on its origin square
        if mv.to == Sq::new(7, 0) { b.castling[0] = false; }
        if mv.to == Sq::new(0, 0) { b.castling[1] = false; }
        if mv.to == Sq::new(7, 7) { b.castling[2] = false; }
        if mv.to == Sq::new(0, 7) { b.castling[3] = false; }

        b.turn = b.turn.flip();
        b
    }

    fn leaves_in_check(&self, mv: Move) -> bool {
        self.apply_move(mv).is_in_check(self.turn)
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        self.find_king(color)
            .map(|sq| self.is_attacked(sq, color.flip()))
            .unwrap_or(false)
    }

    fn find_king(&self, color: Color) -> Option<Sq> {
        (0..8u8).flat_map(|r| (0..8u8).map(move |f| Sq::new(f, r)))
            .find(|&sq| self.get(sq) == Some((color, Piece::King)))
    }

    fn is_attacked(&self, sq: Sq, by: Color) -> bool {
        // Knights
        for (df, dr) in [(-2i8,-1i8),(-2,1),(-1,-2),(-1,2),(1,-2),(1,2),(2,-1),(2,1)] {
            if let Some(s) = sq.offset(df, dr) {
                if self.get(s) == Some((by, Piece::Knight)) { return true; }
            }
        }
        // King
        for df in -1i8..=1 {
            for dr in -1i8..=1 {
                if df == 0 && dr == 0 { continue; }
                if let Some(s) = sq.offset(df, dr) {
                    if self.get(s) == Some((by, Piece::King)) { return true; }
                }
            }
        }
        // Rook / Queen (straights)
        for (df, dr) in [(1i8,0i8),(-1,0),(0,1),(0,-1)] {
            let mut s = sq;
            loop {
                match s.offset(df, dr) {
                    None => break,
                    Some(ns) => {
                        s = ns;
                        match self.get(s) {
                            None => {}
                            Some((c, Piece::Rook))  | Some((c, Piece::Queen)) if c == by => return true,
                            Some(_) => break,
                        }
                    }
                }
            }
        }
        // Bishop / Queen (diagonals)
        for (df, dr) in [(1i8,1i8),(1,-1),(-1,1),(-1,-1)] {
            let mut s = sq;
            loop {
                match s.offset(df, dr) {
                    None => break,
                    Some(ns) => {
                        s = ns;
                        match self.get(s) {
                            None => {}
                            Some((c, Piece::Bishop)) | Some((c, Piece::Queen)) if c == by => return true,
                            Some(_) => break,
                        }
                    }
                }
            }
        }
        // Pawns
        let pawn_dr: i8 = if by == Color::White { 1 } else { -1 };
        for df in [-1i8, 1i8] {
            if let Some(s) = sq.offset(df, -pawn_dr) {
                if self.get(s) == Some((by, Piece::Pawn)) { return true; }
            }
        }
        false
    }

    fn pseudo_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        for rank in 0..8u8 {
            for file in 0..8u8 {
                let sq = Sq::new(file, rank);
                if let Some((color, piece)) = self.get(sq) {
                    if color == self.turn {
                        self.piece_pseudo(sq, color, piece, &mut moves);
                    }
                }
            }
        }
        moves
    }

    fn piece_pseudo(&self, sq: Sq, color: Color, piece: Piece, out: &mut Vec<Move>) {
        match piece {
            Piece::Pawn   => self.pawn_pseudo(sq, color, out),
            Piece::Knight => {
                for (df, dr) in [(-2i8,-1i8),(-2,1),(-1,-2),(-1,2),(1,-2),(1,2),(2,-1),(2,1)] {
                    if let Some(to) = sq.offset(df, dr) {
                        if self.get(to).map_or(true, |(c,_)| c != color) {
                            out.push(Move { from: sq, to, promotion: None });
                        }
                    }
                }
            }
            Piece::Bishop => self.sliding(sq, color, &[(1,1),(1,-1),(-1,1),(-1,-1)], out),
            Piece::Rook   => self.sliding(sq, color, &[(1,0),(-1,0),(0,1),(0,-1)], out),
            Piece::Queen  => {
                self.sliding(sq, color, &[(1,0),(-1,0),(0,1),(0,-1)], out);
                self.sliding(sq, color, &[(1,1),(1,-1),(-1,1),(-1,-1)], out);
            }
            Piece::King   => self.king_pseudo(sq, color, out),
        }
    }

    fn pawn_pseudo(&self, sq: Sq, color: Color, out: &mut Vec<Move>) {
        let dir: i8         = if color == Color::White { 1 } else { -1 };
        let start_rank: u8  = if color == Color::White { 1 } else { 6 };
        let promo_rank: u8  = if color == Color::White { 7 } else { 0 };

        let push_promos = |to: Sq, out: &mut Vec<Move>| {
            for &p in &[Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                out.push(Move { from: sq, to, promotion: Some(p) });
            }
        };

        // Single push
        if let Some(to) = sq.offset(0, dir) {
            if self.get(to).is_none() {
                if to.rank == promo_rank {
                    push_promos(to, out);
                } else {
                    out.push(Move { from: sq, to, promotion: None });
                    // Double push from starting rank
                    if sq.rank == start_rank {
                        if let Some(to2) = sq.offset(0, 2 * dir) {
                            if self.get(to2).is_none() {
                                out.push(Move { from: sq, to: to2, promotion: None });
                            }
                        }
                    }
                }
            }
        }

        // Captures (including en passant)
        for df in [-1i8, 1i8] {
            if let Some(to) = sq.offset(df, dir) {
                let occupied_by_enemy = self.get(to).map_or(false, |(c,_)| c != color);
                let is_ep = self.en_passant == Some(to);
                if occupied_by_enemy || is_ep {
                    if to.rank == promo_rank {
                        push_promos(to, out);
                    } else {
                        out.push(Move { from: sq, to, promotion: None });
                    }
                }
            }
        }
    }

    fn sliding(&self, sq: Sq, color: Color, dirs: &[(i8,i8)], out: &mut Vec<Move>) {
        for &(df, dr) in dirs {
            let mut s = sq;
            loop {
                match s.offset(df, dr) {
                    None => break,
                    Some(ns) => {
                        s = ns;
                        match self.get(s) {
                            None => out.push(Move { from: sq, to: s, promotion: None }),
                            Some((c, _)) if c != color => {
                                out.push(Move { from: sq, to: s, promotion: None });
                                break;
                            }
                            Some(_) => break,
                        }
                    }
                }
            }
        }
    }

    fn king_pseudo(&self, sq: Sq, color: Color, out: &mut Vec<Move>) {
        for df in -1i8..=1 {
            for dr in -1i8..=1 {
                if df == 0 && dr == 0 { continue; }
                if let Some(to) = sq.offset(df, dr) {
                    if self.get(to).map_or(true, |(c,_)| c != color) {
                        out.push(Move { from: sq, to, promotion: None });
                    }
                }
            }
        }

        let r = if color == Color::White { 0u8 } else { 7u8 };
        let (ks, qs) = if color == Color::White { (0, 1) } else { (2, 3) };
        let enemy = color.flip();

        if sq != Sq::new(4, r) || self.is_in_check(color) { return; }

        // Kingside castling
        if self.castling[ks]
            && self.get(Sq::new(5, r)).is_none()
            && self.get(Sq::new(6, r)).is_none()
            && !self.is_attacked(Sq::new(5, r), enemy)
            && !self.is_attacked(Sq::new(6, r), enemy)
        {
            out.push(Move { from: sq, to: Sq::new(6, r), promotion: None });
        }

        // Queenside castling
        if self.castling[qs]
            && self.get(Sq::new(3, r)).is_none()
            && self.get(Sq::new(2, r)).is_none()
            && self.get(Sq::new(1, r)).is_none()
            && !self.is_attacked(Sq::new(3, r), enemy)
            && !self.is_attacked(Sq::new(2, r), enemy)
        {
            out.push(Move { from: sq, to: Sq::new(2, r), promotion: None });
        }
    }

    pub fn is_checkmate(&self) -> bool {
        self.is_in_check(self.turn) && self.legal_moves().is_empty()
    }

    pub fn is_stalemate(&self) -> bool {
        !self.is_in_check(self.turn) && self.legal_moves().is_empty()
    }
}
