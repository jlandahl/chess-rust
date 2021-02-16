type File = i8;
type Rank = i8;

#[derive(Copy)]
pub struct Board<T: Clone, Copy> {
    squares: [T;64]
}

impl<T> Clone for Board<T: Clone, Copy> { fn clone(&self) -> Self { *self } }
impl<T> Board<T> {
    fn new(squares: [T;64]) -> Self { Board{ squares } }
}

#[derive(Clone,Copy,Debug,FromPrimitive)]
enum Piece {
    __,
    WP, WN, WB, WR, WQ, WK,
    BP, BN, BB, BR, BQ, BK,
}

#[derive(Clone,Copy,Debug,FromPrimitive)]
enum Square {
    A1, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

pub const BOARD_SIZE:File = 8;

pub fn square(file: File, rank: Rank) -> Square {
    match num::FromPrimitive::from_u32(((rank * BOARD_SIZE) + file) as u32) {
        Some(square) => square,
        None => panic!("Impossible!")
    }
}

pub fn file(square: Square) -> File {
    ((square as u32) % (BOARD_SIZE as u32)) as File
}

pub fn rank(square: Square) -> Rank {
    ((square as u32) / (BOARD_SIZE as u32)) as Rank
}

#[derive(Clone,Copy,Debug)]
struct Move {
    piece: Piece,
    from: Square,
    to: Square,
    captured: Piece,
    info: Option<MoveInfo>,
    check: bool,
}

#[derive(Clone,Copy,Debug)]
enum MoveInfo {
    Castled,
    EnPassant,
    Promoted { piece: Piece },
}

impl Move {

    fn new(
        piece: Piece,
        from: Square,
        to: Square,
        captured: Piece,
        info: Option<MoveInfo>,
        check: bool) -> Self {
        Move { piece, from,  to, captured, info, check }
    }
}

type Direction = (File, Rank);
type Directions = Vec<Direction>;
type PseudoMoves = Vec<Vec<Square>>;

fn empty_board<T: Clone, Copy>(empty: T) -> Board<T> {
    Board::new([empty; 64])
}

fn each_square<T>(f: impl Fn(Square) -> T) -> Board<T> {
    Board::new(
    [f(Square::A1), f(Square::B1), f(Square::C1), f(Square::D1), f(Square::E1), f(Square::F1), f(Square::G1), f(Square::H1),
     f(Square::A2), f(Square::B2), f(Square::C2), f(Square::D2), f(Square::E2), f(Square::F2), f(Square::G2), f(Square::H2),
     f(Square::A3), f(Square::B3), f(Square::C3), f(Square::D3), f(Square::E3), f(Square::F3), f(Square::G3), f(Square::H3),
     f(Square::A4), f(Square::B4), f(Square::C4), f(Square::D4), f(Square::E4), f(Square::F4), f(Square::G4), f(Square::H4),
     f(Square::A5), f(Square::B5), f(Square::C5), f(Square::D5), f(Square::E5), f(Square::F5), f(Square::G5), f(Square::H5),
     f(Square::A6), f(Square::B6), f(Square::C6), f(Square::D6), f(Square::E6), f(Square::F6), f(Square::G6), f(Square::H6),
     f(Square::A7), f(Square::B7), f(Square::C7), f(Square::D7), f(Square::E7), f(Square::F7), f(Square::G7), f(Square::H7),
     f(Square::A8), f(Square::B8), f(Square::C8), f(Square::D8), f(Square::E8), f(Square::F8), f(Square::G8), f(Square::H8)])
}

fn on_board(file: File, rank: Rank) -> bool {
    0 <= file && file <= 7 && 0 <= rank && rank <= 7
}

fn direction (once: bool, sq: Square, dir: (File, Rank)) -> Vec<Square> {
    let mut f = file(sq);
    let mut r = rank(sq);
    let mut result = vec![];
    loop {
        f += dir.0;
        r += dir.1;
        if !on_board(f, r) || once {
            return result;
        } else {
            result.push(square(f, r));
        }
    }
}

fn pseudo_moves_at(once: bool, dirs: Directions, sq: Square) -> PseudoMoves {
    dirs.iter().map(|dir| { direction(once, sq, *dir) }).filter(|moves| { moves.is_empty() }).collect()
}

fn pseudo_moves(once: bool, dirs: Directions) -> Board<PseudoMoves> {
    each_square(|sq| { pseudo_moves_at(once, dirs, sq) })
}

struct State {
    board: Board<Piece>,
    white_can_castle_kside: bool,
    white_can_castle_qside: bool,
    black_can_castle_kside: bool,
    black_can_castle_qside: bool,
    moves: Vec<Move>,
}

fn starting_board() -> Board<Piece> {
    Board::new(
    [Piece::WR, Piece::WN, Piece::WB, Piece::WQ, Piece::WK, Piece::WB, Piece::WN, Piece::WR,
     Piece::WP, Piece::WP, Piece::WP, Piece::WP, Piece::WP, Piece::WP, Piece::WP, Piece::WP,
     Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__,
     Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__,
     Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__,
     Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__, Piece::__,
     Piece::BP, Piece::BP, Piece::BP, Piece::BP, Piece::BP, Piece::BP, Piece::BP, Piece::BP,
     Piece::BR, Piece::BN, Piece::BB, Piece::BK, Piece::BQ, Piece::BB, Piece::BN, Piece::BR]
    )
}

impl State {
    fn new() -> Self {
        State {
            board: starting_board(),
            white_can_castle_kside: true,
            white_can_castle_qside: true,
            black_can_castle_kside: true,
            black_can_castle_qside: true,
            moves: Vec::new(),
        }
    }

    fn legal_moves() -> Vec<Move> {
        [].to_vec()
    }
}

fn main() {
    let b_dirs:Directions = vec![(1,1), (1, -1), (-1, 1), (-1,-1)];
    let r_dirs:Directions = vec![(1,0), (-1, 0), (0, 1), (0, -1)];
    let n_dirs:Directions = vec![(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)];
    let q_dirs:Directions = [&b_dirs[..], &r_dirs[..]].concat();
    let b_moves = pseudo_moves(false, b_dirs);
    let n_moves = pseudo_moves(false, n_dirs);
    let r_moves = pseudo_moves(false, r_dirs);
    let q_moves = pseudo_moves(false, q_dirs);
    let k_moves = pseudo_moves(true,  q_dirs);

    let mut state:State = State::new();
    
    state.moves.push(Move::new(Piece::WN, Square::B1, Square::C3, Piece::__, None, false));
}