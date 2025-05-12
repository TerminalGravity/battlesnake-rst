use crate::game_state::{Coord, GameState, Move};
use log::debug;

// Default health threshold below which the snake will seek food.
const DEFAULT_FOOD_THRESHOLD: u32 = 50;

// Determines if the snake should actively seek food based on health.
pub fn should_seek_food(game_state: &GameState) -> bool {
    let threshold = std::env::var("FOOD_THRESHOLD")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(DEFAULT_FOOD_THRESHOLD);

    let should_seek = game_state.you.health < threshold;
    if should_seek {
        debug!(
            "Game {} Turn {}: Health ({}) below threshold ({}), seeking food.",
            game_state.game.id,
            game_state.turn,
            game_state.you.health,
            threshold
        );
    }
    should_seek
}

// Finds the safe move that leads closest to the nearest food item.
// Returns None if no food exists or no safe moves are provided.
pub fn find_move_to_closest_food(game_state: &GameState, safe_moves: &[Move]) -> Option<Move> {
    if game_state.board.food.is_empty() || safe_moves.is_empty() {
        return None;
    }

    let head = &game_state.you.head;

    // Find the coordinates of the closest food item
    let closest_food_coord = game_state.board.food.iter()
        .min_by_key(|food_coord| manhattan_distance(head, food_coord))
        .cloned(); // Clone the Option<Coord>

    // If no food found (should not happen if board.food wasn't empty, but check anyway)
    let target_food = match closest_food_coord {
        Some(coord) => coord,
        None => return None,
    };
    debug!(
        "Game {} Turn {}: Closest food at ({}, {}).",
        game_state.game.id, game_state.turn, target_food.x, target_food.y
    );


    // Find which safe move gets us closest (minimum Manhattan distance) to the target food
    let best_move = safe_moves.iter()
        .min_by_key(|&&m| {
            let next_pos = head.apply_move(m);
            manhattan_distance(&next_pos, &target_food)
        })
        .cloned(); // Clone the Option<Move>

    if let Some(chosen_move) = best_move {
         debug!(
            "Game {} Turn {}: Chose move {} towards food.",
            game_state.game.id, game_state.turn, chosen_move.as_str()
        );
    }

    best_move
}

// Calculates the Manhattan distance between two coordinates.
fn manhattan_distance(a: &Coord, b: &Coord) -> u32 {
    ((a.x - b.x).abs() + (a.y - b.y).abs()) as u32
} 