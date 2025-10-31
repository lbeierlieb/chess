use bevy::{prelude::*, sprite::Anchor};
use bevy_modern_pixel_camera::prelude::*;
use gamelogic::{
    coordinates::Position,
    game::Game,
    moves,
    pieces::{self, PieceType},
};
use std::f32::consts::PI;

pub mod gamelogic;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelCameraPlugin)
        .insert_resource(ChessGame::default())
        .add_systems(Startup, initialize_rendering)
        .insert_resource(MouseBoardPosition::default())
        .add_systems(
            Update,
            (
                update_mouse_board_position,
                mouse_click_handler,
                (rotate_selected_marker, animate_possible_moves),
            )
                .chain(),
        )
        .add_systems(Update, (move_light))
        .add_observer(new_selection_handler)
        .add_observer(try_move_handler)
        .add_observer(check_winner)
        .add_observer(successful_move_handler)
        .run();
}

#[derive(Resource)]
struct ChessGame {
    game: Game,
    selected_tile: Option<Position>,
}

impl Default for ChessGame {
    fn default() -> Self {
        Self {
            game: Game::new(),
            selected_tile: None,
        }
    }
}

#[derive(Component)]
struct PossibleMoveHighlight {
    base_height: f32,
}

#[derive(Component)]
struct PieceMarker {
    pos: Position,
}

fn initialize_rendering(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<ChessGame>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(8.0, 20.0, 8.).looking_at(Vec3::new(8., 0., -8.), Vec3::Y),
    ));

    commands.spawn((
        PointLight {
            intensity: 5_000_000.0,
            ..default()
        },
        Transform::default(),
    ));

    commands.spawn((
        SceneRoot(asset_server.load("board.glb#Scene0")),
        Transform::from_xyz(8.0, 0., -8.0).with_rotation(Quat::from_axis_angle(Vec3::Y, PI * 0.5)),
    ));

    let king_white = asset_server.load("king_white.glb#Scene0");
    let king_black = asset_server.load("king_black.glb#Scene0");
    let queen_white = asset_server.load("queen_white.glb#Scene0");
    let queen_black = asset_server.load("queen_black.glb#Scene0");
    let rook_white = asset_server.load("rook_white.glb#Scene0");
    let rook_black = asset_server.load("rook_black.glb#Scene0");
    let bishop_white = asset_server.load("bishop_white.glb#Scene0");
    let bishop_black = asset_server.load("bishop_black.glb#Scene0");
    let knight_white = asset_server.load("knight_white.glb#Scene0");
    let knight_black = asset_server.load("knight_black.glb#Scene0");
    let pawn_white = asset_server.load("pawn_white.glb#Scene0");
    let pawn_black = asset_server.load("pawn_black.glb#Scene0");

    for x in 0..8 {
        for y in 0..8 {
            if let Some(piece) = game.game.piece_at(Position::new(x, y)) {
                let scene = match (piece.piece_type, piece.color) {
                    (PieceType::King, pieces::Color::White) => king_white.clone(),
                    (PieceType::King, pieces::Color::Black) => king_black.clone(),
                    (PieceType::Queen, pieces::Color::White) => queen_white.clone(),
                    (PieceType::Queen, pieces::Color::Black) => queen_black.clone(),
                    (PieceType::Rook, pieces::Color::White) => rook_white.clone(),
                    (PieceType::Rook, pieces::Color::Black) => rook_black.clone(),
                    (PieceType::Bishop, pieces::Color::White) => bishop_white.clone(),
                    (PieceType::Bishop, pieces::Color::Black) => bishop_black.clone(),
                    (PieceType::Knight, pieces::Color::White) => knight_white.clone(),
                    (PieceType::Knight, pieces::Color::Black) => knight_black.clone(),
                    (PieceType::Pawn, pieces::Color::White) => pawn_white.clone(),
                    (PieceType::Pawn, pieces::Color::Black) => pawn_black.clone(),
                };
                let y_rot = if piece.piece_type == PieceType::Knight
                    && piece.color == pieces::Color::Black
                {
                    PI
                } else {
                    0.
                };
                commands.spawn((
                    SceneRoot(scene),
                    Transform::from_translation(Vec3::new(
                        (x * 2 + 1) as f32,
                        0.,
                        (y as f32) * (-2.) - 1.,
                    ))
                    .with_scale(Vec3::new(0.9, 0.9, 0.9))
                    .with_rotation(Quat::from_axis_angle(Vec3::Y, y_rot)),
                    PieceMarker {
                        pos: Position::new(x, y),
                    },
                ));
            }
        }
    }
}

fn move_light(mut query: Query<&mut Transform, With<PointLight>>, time: Res<Time>) {
    let center = Vec3::new(8., 8., -8.);
    let distance = 4.;
    let rot = Quat::from_axis_angle(Vec3::Y, time.elapsed_secs() * PI * 0.1);
    let pos = center + rot.mul_vec3(Vec3::new(0., 0., distance));
    for mut transform in &mut query {
        transform.translation = pos;
    }
}

#[derive(Component)]
struct SelectedMarker {}

fn rotate_selected_marker(mut query: Query<&mut Transform, With<SelectedMarker>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_axis_angle(Vec3::Y, time.elapsed_secs() * PI * 2.);
    }
}

fn animate_possible_moves(
    mut query: Query<(&mut Transform, &mut PossibleMoveHighlight)>,
    time: Res<Time>,
) {
    for (mut transform, mut highlight) in &mut query {
        let individual_offset = 1.0 * (transform.translation.x - transform.translation.z) / 30.;
        transform.rotation = Quat::from_axis_angle(
            Vec3::Y,
            0.25 * PI * ((time.elapsed_secs() + individual_offset) * PI * 0.5).sin(),
        );
        transform.translation.y = highlight.base_height
            + 0.1
            + 0.1 * ((time.elapsed_secs() + individual_offset) * PI * 1.5).sin();
    }
}
#[derive(Resource, Default)]
struct MouseBoardPosition {
    field: Option<(u8, u8)>,
}

fn update_mouse_board_position(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut mouse_board_position: ResMut<MouseBoardPosition>,
) {
    let window = window.single().unwrap();
    let (camera, camera_transform) = camera.single().unwrap();

    if let Some(ray) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
    {
        if ray.direction.y > -0.0001 {
            mouse_board_position.field = None;
            return;
        }
        let t = -ray.origin.y / ray.direction.y;
        let intersect = ray.origin + ray.direction * t;
        if intersect.x < 0. || intersect.z > 0. {
            mouse_board_position.field = None;
            return;
        }
        let x = intersect.x as u64 / 2;
        let y = (-intersect.z) as u64 / 2;
        mouse_board_position.field = if x <= 7 && y <= 7 {
            Some((x as u8, y as u8))
        } else {
            None
        };
    }
}

#[derive(Event)]
struct SelectionChangedEvent {}

fn new_selection_handler(
    event: On<SelectionChangedEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    selected_marker: Query<Entity, With<SelectedMarker>>,
    highlights: Query<Entity, With<PossibleMoveHighlight>>,
    game: Res<ChessGame>,
) {
    for entity in selected_marker {
        commands.entity(entity).despawn();
    }
    for entity in highlights {
        commands.entity(entity).despawn();
    }

    if let Some(pos) = game.selected_tile {
        commands.spawn((
            SceneRoot(asset_server.load("selected_piece.glb#Scene0")),
            Transform::from_translation(Vec3::new(
                (pos.x * 2 + 1) as f32,
                0.,
                (pos.y as f32) * (-2.) - 1.,
            )),
            SelectedMarker {},
        ));
    }

    let possible_moves: Vec<Position> = game
        .selected_tile
        .iter()
        .flat_map(|&pos| moves::valid_destinations(pos, &game.game))
        .collect();

    let handle = asset_server.load("possible_move.glb#Scene0");
    for pos in possible_moves {
        let base_height = match game.game.piece_at(pos).map(|piece| piece.piece_type) {
            Some(PieceType::Pawn) => 2.6,
            Some(PieceType::Knight) => 2.8,
            Some(PieceType::Bishop) => 3.5,
            Some(PieceType::Rook) => 2.5,
            Some(PieceType::Queen) => 3.8,
            Some(PieceType::King) => 4.1,
            None => 0.2,
        };
        commands.spawn((
            SceneRoot(handle.clone()),
            Transform::from_translation(Vec3::new(
                (pos.x * 2 + 1) as f32,
                0.,
                (pos.y as f32) * (-2.) - 1.,
            )),
            PossibleMoveHighlight { base_height },
        ));
    }
}

#[derive(Event)]
struct TryMoveEvent {
    origin: Position,
    destination: Position,
}

fn try_move_handler(event: On<TryMoveEvent>, mut game: ResMut<ChessGame>, mut commands: Commands) {
    let move_req = moves::MoveRequest::new(event.origin, event.destination, None);
    if let Some(new_game) = game.game.perform_move_request(move_req) {
        game.game = new_game;
        commands.trigger(SuccessfulMoveEvent {});
    }
}

#[derive(Event)]
struct SuccessfulMoveEvent {}

fn check_winner(_: On<SuccessfulMoveEvent>, game: Res<ChessGame>) {
    if let Some(winner) = game.game.winner() {
        println!("The winner is {:?}", winner);
    }
}

fn successful_move_handler(
    _: On<SuccessfulMoveEvent>,
    game: Res<ChessGame>,
    mut pieces: Query<(&mut Transform, &mut PieceMarker)>,
) {
    // Safety: We are in successful_move_handler, so there has to be a last move.
    let last_move = game.game.last_move.unwrap();
    let moves = match last_move {
        moves::Move::NormalMove(normal_move) => {
            vec![(normal_move.origin, normal_move.destination)]
        }
        moves::Move::EnPassante(en_passante) => {
            vec![(en_passante.origin, en_passante.destination)]
        }
        moves::Move::Castling(castling) => vec![
            (castling.king_origin, castling.king_destination),
            (castling.rook_origin, castling.rook_destination),
        ],
        moves::Move::Promotion(_) => todo!(),
    };
    let thrown = match last_move {
        moves::Move::NormalMove(normal_move) => {
            normal_move.throwing.map(|_| normal_move.destination)
        }
        moves::Move::EnPassante(en_passante) => Some(en_passante.throwing.0),
        moves::Move::Castling(_) => None,
        moves::Move::Promotion(_) => None,
    };

    if let Some(throw_pos) = thrown {
        for (mut transform, mut marker) in pieces.iter_mut() {
            if marker.pos == throw_pos {
                // TODO despawn instead
                transform.translation.y = -5.;
            }
        }
    }
    for (mut transform, mut marker) in pieces.iter_mut() {
        for &(origin, destination) in moves.iter() {
            if marker.pos == origin {
                marker.pos = destination;
                transform.translation.x = destination.x as f32 * 2. + 1.;
                transform.translation.z = -(destination.y as f32 * 2. + 1.);
            }
        }
    }
}

fn mouse_click_handler(
    mouse_button_input_reader: Res<ButtonInput<MouseButton>>,
    mouse_board_position: Res<MouseBoardPosition>,
    asset_server: Res<AssetServer>,
    mut game: ResMut<ChessGame>,
    mut commands: Commands,
    mut pieces: Query<&mut Transform, With<PieceMarker>>,
) {
    if !mouse_button_input_reader.just_pressed(MouseButton::Left) {
        return;
    }

    let selected_movable = mouse_board_position.field.and_then(|(x, y)| {
        let pos = Position::new(x, y);
        game.game
            .piece_at(pos)
            .and_then(|piece| {
                if piece.color == game.game.active_color() {
                    Some(())
                } else {
                    None
                }
            })
            .map(|_| pos)
    });

    if selected_movable == game.selected_tile {
        // click on same tile as last time, nothing today
        return;
    }

    if let Some(pos_moveable) = selected_movable {
        // clicked on friendly field, showing possible moves
        game.selected_tile = selected_movable;
        commands.trigger(SelectionChangedEvent {});
    } else if let (Some(origin), Some((dest_x, dest_y))) =
        (game.selected_tile, mouse_board_position.field)
    {
        // previously selected a tile, now clicked on another field. Try to do the move.
        commands.trigger(TryMoveEvent {
            origin,
            destination: Position::new(dest_x, dest_y),
        });
        // either the move succeeds and the board changes or the user clicked on a tile that is
        // unreachable for the selected piece. In both cases, we deselect the current tile.
        game.selected_tile = None;
        commands.trigger(SelectionChangedEvent {});
    }
}
