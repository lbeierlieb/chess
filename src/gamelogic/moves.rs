use std::ops::ControlFlow;

use crate::gamelogic::coordinates::Direction;

use super::{
    coordinates::Position,
    game::Game,
    pieces::{Color, Piece, PieceType},
};

#[derive(Debug, Clone, Copy)]
pub enum Move {
    NormalMove(NormalMove),
    EnPassante(EnPassante),
    Castling(Castling),
    Promotion(Promotion),
}

#[derive(Debug, Clone, Copy)]
pub struct NormalMove {
    pub origin: Position,
    pub destination: Position,
    pub throwing: Option<Piece>,
}

#[derive(Debug, Clone, Copy)]
pub struct EnPassante {
    pub origin: Position,
    pub destination: Position,
    pub throwing: (Position, Piece),
}

#[derive(Debug, Clone, Copy)]
pub struct Castling {
    pub king_origin: Position,
    pub king_destination: Position,
    pub rook_origin: Position,
    pub rook_destination: Position,
}

#[derive(Debug, Clone, Copy)]
pub struct Promotion {
    pub origin: Position,
    pub destination: Position,
    pub new_piece: Piece,
}

#[derive(Debug, Clone, Copy)]
pub struct MoveRequest {
    pub origin: Position,
    pub destination: Position,
    pub promotion: Option<Piece>,
}

impl MoveRequest {
    pub fn new(origin: Position, destination: Position, promotion: Option<Piece>) -> Self {
        Self {
            origin,
            destination,
            promotion,
        }
    }

    pub fn to_move(&self, game: &Game) -> Option<Move> {
        valid_destinations_with_special_cases(self.origin, game)
            .into_iter()
            .filter(|mov| match mov {
                Move::NormalMove(normal_move) => {
                    normal_move.origin == self.origin && normal_move.destination == self.destination
                }
                Move::EnPassante(en_passante) => {
                    en_passante.origin == self.origin && en_passante.destination == self.destination
                }
                Move::Castling(castling) => {
                    castling.king_origin == self.origin
                        && castling.king_destination == self.destination
                }
                Move::Promotion(promotion) => {
                    promotion.origin == self.origin
                        && promotion.destination == self.destination
                        && Some(promotion.new_piece) == self.promotion
                }
            })
            .next()
    }
}

pub fn valid_destinations(origin: Position, game: &Game) -> Vec<Position> {
    valid_destinations_with_special_cases(origin, game)
        .into_iter()
        .map(|mov| match mov {
            Move::NormalMove(normal_move) => normal_move.destination,
            Move::EnPassante(en_passante) => en_passante.destination,
            Move::Castling(castling) => castling.king_destination,
            Move::Promotion(promotion) => promotion.destination,
        })
        .collect()
}

fn valid_destinations_with_special_cases(origin: Position, game: &Game) -> Vec<Move> {
    let piece = match game.piece_at(origin) {
        Some(piece) => piece,
        None => return Vec::new(),
    };
    match piece.piece_type {
        super::pieces::PieceType::King => {
            let mut destinations = wrap_as_normal(
                destinations(origin, &Direction::all(), 1, game),
                origin,
                game,
            );
            destinations.append(&mut castling_destinations(origin, game));
            destinations
        }
        super::pieces::PieceType::Queen => wrap_as_normal(
            destinations(origin, &Direction::all(), 7, game),
            origin,
            game,
        ),
        super::pieces::PieceType::Rook => wrap_as_normal(
            destinations(origin, &Direction::all_non_diagonal(), 7, game),
            origin,
            game,
        ),
        super::pieces::PieceType::Bishop => wrap_as_normal(
            destinations(origin, &Direction::all_diagonal(), 7, game),
            origin,
            game,
        ),
        super::pieces::PieceType::Knight => {
            wrap_as_normal(knight_destinations(origin, game), origin, game)
        }
        super::pieces::PieceType::Pawn => pawn_destinations(origin, game),
    }
    .into_iter()
    .filter(|mov| {
        !game
            .perform_move(*mov)
            .unwrap()
            .is_king_in_check(piece.color)
    })
    .collect()
}

fn wrap_as_normal(positions: Vec<Position>, origin: Position, game: &Game) -> Vec<Move> {
    positions
        .into_iter()
        .map(|pos| {
            Move::NormalMove(NormalMove {
                origin,
                destination: pos,
                throwing: game.piece_at(pos),
            })
        })
        .collect()
}

fn castling_destinations(origin: Position, game: &Game) -> Vec<Move> {
    let mut destinations = vec![];
    let king = game.piece_at(origin).unwrap();

    if king.has_moved {
        return destinations;
    }

    let color = king.color;
    let expected_pos = match color {
        Color::White => Position::from_str("E1"),
        Color::Black => Position::from_str("E8"),
    };

    if expected_pos != origin {
        return destinations;
    }

    if let Some(d) = castling_left(origin, game) {
        destinations.push(d);
    }
    if let Some(d) = castling_right(origin, game) {
        destinations.push(d);
    }

    destinations
}

fn castling_left(origin: Position, game: &Game) -> Option<Move> {
    if game
        .piece_at(origin.moved(Direction::West, 1).unwrap())
        .is_some()
    {
        return None;
    }
    if game
        .piece_at(origin.moved(Direction::West, 2).unwrap())
        .is_some()
    {
        return None;
    }
    if game
        .piece_at(origin.moved(Direction::West, 3).unwrap())
        .is_some()
    {
        return None;
    }
    if let Some(piece) = game.piece_at(origin.moved(Direction::West, 4).unwrap()) {
        if piece.piece_type == PieceType::Rook && !piece.has_moved {
            return Some(Move::Castling(Castling {
                king_origin: origin,
                king_destination: origin.moved(Direction::West, 2).unwrap(),
                rook_origin: origin.moved(Direction::West, 4).unwrap(),
                rook_destination: origin.moved(Direction::West, 1).unwrap(),
            }));
        }
    }
    None
}

fn castling_right(origin: Position, game: &Game) -> Option<Move> {
    if game
        .piece_at(origin.moved(Direction::East, 1).unwrap())
        .is_some()
    {
        return None;
    }
    if game
        .piece_at(origin.moved(Direction::East, 2).unwrap())
        .is_some()
    {
        return None;
    }
    if let Some(piece) = game.piece_at(origin.moved(Direction::East, 3).unwrap()) {
        if piece.piece_type == PieceType::Rook && !piece.has_moved {
            return Some(Move::Castling(Castling {
                king_origin: origin,
                king_destination: origin.moved(Direction::East, 2).unwrap(),
                rook_origin: origin.moved(Direction::East, 3).unwrap(),
                rook_destination: origin.moved(Direction::East, 1).unwrap(),
            }));
        }
    }
    None
}

fn pawn_destinations(origin: Position, game: &Game) -> Vec<Move> {
    let mut destinations = vec![];

    let color = game.piece_at(origin).unwrap().color;
    let has_moved = game.piece_at(origin).unwrap().has_moved;
    let dir = match color {
        Color::White => Direction::North,
        Color::Black => Direction::South,
    };
    if let Some(one_step_forward) = origin.moved(dir, 1) {
        match game.piece_at(one_step_forward) {
            None => {
                destinations.push(Move::NormalMove(NormalMove {
                    origin,
                    destination: one_step_forward,
                    throwing: None,
                }));

                if !has_moved {
                    if let Some(two_step_forward) = origin.moved(dir, 2) {
                        match game.piece_at(two_step_forward) {
                            None => destinations.push(Move::NormalMove(NormalMove {
                                origin,
                                destination: two_step_forward,
                                throwing: None,
                            })),
                            _ => {}
                        }
                    }
                };
            }
            _ => {}
        };
    }

    for side_dir in vec![Direction::West, Direction::East] {
        if let Some(forward_and_side) = origin.moved(dir, 1).and_then(|p| p.moved(side_dir, 1)) {
            match game.piece_at(forward_and_side) {
                None => {}
                Some(piece) if piece.color == color => {}
                Some(piece) if piece.color != color => {
                    destinations.push(Move::NormalMove(NormalMove {
                        origin,
                        destination: forward_and_side,
                        throwing: Some(piece),
                    }));
                }
                _ => unreachable!(),
            };
        }
    }

    for side_dir in vec![Direction::West, Direction::East] {
        if let Some(side_pos) = origin.moved(side_dir, 1) {
            if let Some(piece) = game.piece_at(side_pos) {
                if piece.piece_type != PieceType::Pawn || piece.color == color {
                    continue;
                }

                // Safety: if there is an enemy pawn next to one of our pawns, moves must have happened
                if let Move::NormalMove(normal_move) = game.last_move.unwrap() {
                    // Safety: checked existence of position next to us before with the if let
                    if normal_move.destination == side_pos
                        && (normal_move.destination.y as i8 - normal_move.origin.y as i8).abs() == 2
                    {
                        destinations.push(Move::EnPassante(EnPassante {
                            origin,
                            destination: side_pos.moved(dir, 1).unwrap(),
                            throwing: (side_pos, piece),
                        }));
                    }
                }
            }
        }
    }

    destinations
}

fn destinations(
    origin: Position,
    directions: &[Direction],
    max_steps: i8,
    game: &Game,
) -> Vec<Position> {
    directions
        .iter()
        .flat_map(|dir| {
            match (1..=max_steps)
                .filter_map(|distance| origin.moved(*dir, distance))
                .try_fold(vec![], |acc, pos| {
                    let color = game.piece_at(origin).unwrap().color;
                    let positions = match is_valid_destination(pos, color, game) {
                        true => {
                            let mut vec = acc.clone();
                            vec.push(pos);
                            vec
                        }
                        false => return ControlFlow::Break(acc),
                    };
                    match is_enemy_at_destination(pos, color, game) {
                        true => ControlFlow::Break(positions),
                        false => ControlFlow::Continue(positions),
                    }
                }) {
                ControlFlow::Continue(positions) => positions,
                ControlFlow::Break(positions) => positions,
            }
        })
        .filter(|pos| is_valid_destination(*pos, game.piece_at(origin).unwrap().color, game))
        .collect()
}

fn knight_destinations(origin: Position, game: &Game) -> Vec<Position> {
    let dirs = Direction::all_non_diagonal();
    dirs.iter()
        .flat_map(|first_dir| {
            dirs.iter().filter_map(|second_dir| {
                if first_dir.is_same_axis(second_dir) {
                    return None;
                }
                origin
                    .moved(*first_dir, 2)
                    .and_then(|pos| pos.moved(*second_dir, 1))
            })
        })
        .filter(|pos| is_valid_destination(*pos, game.piece_at(origin).unwrap().color, game))
        .collect()
}

fn is_valid_destination(destination: Position, color: Color, game: &Game) -> bool {
    match game.piece_at(destination) {
        Some(Piece {
            piece_type: _,
            color: c,
            has_moved: _,
        }) => color != c,
        None => true,
    }
}

fn is_enemy_at_destination(destination: Position, color: Color, game: &Game) -> bool {
    match game.piece_at(destination) {
        Some(Piece {
            piece_type: _,
            color: c,
            has_moved: _,
        }) => color != c,
        None => false,
    }
}
