use std::collections::HashMap;
use std::ops::ControlFlow;

use super::coordinates::Direction;
use super::coordinates::Position;
use super::moves;
use super::moves::Move;
use super::moves::MoveRequest;
use super::pieces::Color;
use super::pieces::Color::*;
use super::pieces::Piece;
use super::pieces::PieceType;
use super::pieces::PieceType::*;

#[derive(Debug, Clone)]
pub struct Game {
    pieces: HashMap<Position, Piece>,
    pub last_move: Option<Move>,
}

impl Game {
    pub fn new() -> Self {
        let mut pieces = HashMap::new();
        pieces.insert(Position::from_str("A1"), Piece::new(Rook, White));
        pieces.insert(Position::from_str("B1"), Piece::new(Knight, White));
        pieces.insert(Position::from_str("C1"), Piece::new(Bishop, White));
        pieces.insert(Position::from_str("D1"), Piece::new(Queen, White));
        pieces.insert(Position::from_str("E1"), Piece::new(King, White));
        pieces.insert(Position::from_str("F1"), Piece::new(Bishop, White));
        pieces.insert(Position::from_str("G1"), Piece::new(Knight, White));
        pieces.insert(Position::from_str("H1"), Piece::new(Rook, White));

        pieces.insert(Position::from_str("A2"), Piece::new(Pawn, White));
        pieces.insert(Position::from_str("B2"), Piece::new(Pawn, White));
        pieces.insert(Position::from_str("C2"), Piece::new(Pawn, White));
        pieces.insert(Position::from_str("D2"), Piece::new(Pawn, White));
        pieces.insert(Position::from_str("E2"), Piece::new(Pawn, White));
        pieces.insert(Position::from_str("F2"), Piece::new(Pawn, White));
        pieces.insert(Position::from_str("G2"), Piece::new(Pawn, White));
        pieces.insert(Position::from_str("H2"), Piece::new(Pawn, White));

        pieces.insert(Position::from_str("A7"), Piece::new(Pawn, Black));
        pieces.insert(Position::from_str("B7"), Piece::new(Pawn, Black));
        pieces.insert(Position::from_str("C7"), Piece::new(Pawn, Black));
        pieces.insert(Position::from_str("D7"), Piece::new(Pawn, Black));
        pieces.insert(Position::from_str("E7"), Piece::new(Pawn, Black));
        pieces.insert(Position::from_str("F7"), Piece::new(Pawn, Black));
        pieces.insert(Position::from_str("G7"), Piece::new(Pawn, Black));
        pieces.insert(Position::from_str("H7"), Piece::new(Pawn, Black));

        pieces.insert(Position::from_str("A8"), Piece::new(Rook, Black));
        pieces.insert(Position::from_str("B8"), Piece::new(Knight, Black));
        pieces.insert(Position::from_str("C8"), Piece::new(Bishop, Black));
        pieces.insert(Position::from_str("D8"), Piece::new(Queen, Black));
        pieces.insert(Position::from_str("E8"), Piece::new(King, Black));
        pieces.insert(Position::from_str("F8"), Piece::new(Bishop, Black));
        pieces.insert(Position::from_str("G8"), Piece::new(Knight, Black));
        pieces.insert(Position::from_str("H8"), Piece::new(Rook, Black));
        Self {
            pieces: pieces,
            last_move: None,
        }
    }

    pub fn piece_at(&self, pos: Position) -> Option<Piece> {
        self.pieces.get(&pos).map(|a| *a)
    }

    pub fn active_color(&self) -> Color {
        self.last_move
            .map(|mov| match mov {
                Move::NormalMove(normal_move) => normal_move.destination,
                Move::EnPassante(en_passante) => en_passante.destination,
                Move::Castling(castling) => castling.king_destination,
                Move::Promotion(promotion) => promotion.destination,
            })
            .map(|destination| self.piece_at(destination).unwrap().color.other())
            .unwrap_or(Color::White)
    }

    pub fn perform_move_request(&self, move_req: MoveRequest) -> Option<Self> {
        if self
            .piece_at(move_req.origin)
            .map(|piece| piece.color != self.active_color())
            .unwrap_or(true)
        {
            return None;
        }

        move_req
            .to_move(self)
            .and_then(|mov| self.perform_move(mov))
    }

    pub fn perform_move(&self, mov: Move) -> Option<Self> {
        match mov {
            Move::NormalMove(normal_move) => {
                let mut pieces = self.pieces.clone();
                let mut moving_piece = pieces.remove(&normal_move.origin).unwrap();
                moving_piece.has_moved = true;
                pieces.insert(normal_move.destination, moving_piece);

                Some(Game {
                    pieces,
                    last_move: Some(mov),
                })
            }
            Move::EnPassante(en_passante) => {
                let mut pieces = self.pieces.clone();
                let moving_piece = pieces.remove(&en_passante.origin).unwrap();
                pieces.insert(en_passante.destination, moving_piece);
                pieces.remove(&en_passante.throwing.0);

                Some(Game {
                    pieces,
                    last_move: Some(mov),
                })
            }
            Move::Castling(castling) => {
                let mut pieces = self.pieces.clone();
                let mut king = pieces.remove(&castling.king_origin).unwrap();
                king.has_moved = true;
                pieces.insert(castling.king_destination, king);
                let mut rook = pieces.remove(&castling.rook_origin).unwrap();
                rook.has_moved = true;
                pieces.insert(castling.rook_destination, rook);

                Some(Game {
                    pieces,
                    last_move: Some(mov),
                })
            }
            Move::Promotion(_promotion) => {
                todo!();
            }
        }
    }

    pub fn winner(&self) -> Option<Color> {
        let active = self.active_color();
        if self
            .pieces
            .iter()
            .filter(|(_, piece)| piece.color == active)
            .all(|(pos, _)| moves::valid_destinations(*pos, self).len() == 0)
        {
            Some(active.other())
        } else {
            None
        }
    }

    pub fn is_king_in_check(&self, color: Color) -> bool {
        let king_pos = self
            .pieces
            .iter()
            .filter(|(_, piece)| piece.piece_type == PieceType::King && piece.color == color)
            .map(|(pos, _)| pos)
            .next()
            .unwrap();
        let enemy_color = color.other();

        let diag_attack = Direction::all_diagonal().iter().any(|dir| {
            (1..8)
                .filter_map(|i| king_pos.moved(*dir, i))
                .try_fold(false, |_, e| match self.piece_at(e) {
                    Some(piece)
                        if piece.piece_type == PieceType::Bishop && piece.color == enemy_color =>
                    {
                        ControlFlow::Break(true)
                    }
                    Some(piece)
                        if piece.piece_type == PieceType::Queen && piece.color == enemy_color =>
                    {
                        ControlFlow::Break(true)
                    }
                    Some(_) => ControlFlow::Break(false),
                    None => ControlFlow::Continue(false),
                })
                .break_value()
                .unwrap_or(false)
        });

        let straight_attack = Direction::all_non_diagonal().iter().any(|dir| {
            (1..8)
                .filter_map(|i| king_pos.moved(*dir, i))
                .try_fold(false, |_, e| match self.piece_at(e) {
                    Some(piece)
                        if piece.piece_type == PieceType::Rook && piece.color == enemy_color =>
                    {
                        ControlFlow::Break(true)
                    }
                    Some(piece)
                        if piece.piece_type == PieceType::Queen && piece.color == enemy_color =>
                    {
                        ControlFlow::Break(true)
                    }
                    Some(_) => ControlFlow::Break(false),
                    None => ControlFlow::Continue(false),
                })
                .break_value()
                .unwrap_or(false)
        });

        let knight_attack = Direction::all_non_diagonal().iter().any(|first_dir| {
            Direction::all_non_diagonal()
                .iter()
                .filter(|second_dir| !first_dir.is_same_axis(*second_dir))
                .any(|second_dir| {
                    king_pos
                        .moved(*first_dir, 2)
                        .and_then(|pos| pos.moved(*second_dir, 1))
                        .and_then(|pos| self.piece_at(pos))
                        .map(|piece| {
                            piece.piece_type == PieceType::Knight && piece.color == enemy_color
                        })
                        .unwrap_or(false)
                })
        });

        let pawn_dir = match color {
            Color::White => Direction::North,
            Color::Black => Direction::South,
        };
        let pawn_attack = vec![Direction::West, Direction::East].iter().any(|dir| {
            king_pos
                .moved(pawn_dir, 1)
                .and_then(|pos| pos.moved(*dir, 1))
                .and_then(|pos| self.piece_at(pos))
                .map(|piece| piece.piece_type == PieceType::Pawn && piece.color == enemy_color)
                .unwrap_or(false)
        });

        let king_attack = Direction::all().iter().any(|dir| {
            king_pos
                .moved(*dir, 1)
                .and_then(|pos| self.piece_at(pos))
                .map(|piece| piece.piece_type == PieceType::King && piece.color == enemy_color)
                .unwrap_or(false)
        });

        diag_attack || straight_attack || knight_attack || pawn_attack || king_attack
    }
}
