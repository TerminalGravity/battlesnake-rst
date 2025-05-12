use crate::game_state::{Coord, GameState, Move};
use log::debug;

// Checks if a potential move `target` leads to a dangerous head-to-head collision.
// Returns true if it's dangerous, false otherwise.
pub fn is_dangerous_head_to_head(
    state: &GameState,
    target: &Coord,
) -> bool {
    let my_id = &state.you.id;
    let my_length = state.you.length;

    for snake in &state.board.snakes {
        // Skip self
        if snake.id == *my_id {
            continue;
        }

        // Skip if snake is shorter or dead (length 0? Check API spec)
        if snake.length == 0 { continue; }

        // Potential head collision square for the opponent
        let their_head = &snake.head;

        // Check if any of the opponent's possible next moves land on our target square
        for &direction in &[Move::Up, Move::Down, Move::Left, Move::Right] {
            let their_potential_target = their_head.apply_move(direction);

            // Is this opponent move valid (within bounds)?
            if !state.board.in_bounds(&their_potential_target) {
                continue;
            }

            // If opponent could move to our target square *and* they are longer or equal length...
            if their_potential_target == *target && snake.length >= my_length {
                debug!(
                    "Game {} Turn {}: Potential head-to-head collision at ({},{}) with snake {} (length {} vs our {}). Avoiding.",
                    state.game.id,
                    state.turn,
                    target.x,
                    target.y,
                    snake.id,
                    snake.length,
                    my_length
                );
                return true; // Dangerous collision detected
            }
        }
    }

    false // No dangerous head-to-head detected for this target square
}

// Basic heuristic to prefer head-to-head if we are longer.
// Note: This is aggressive and doesn't check if the opponent *can* actually move there safely.
// Use with caution, maybe combine with lookahead later.
pub fn is_advantageous_head_to_head(
    state: &GameState,
    target: &Coord,
) -> bool {
     let my_id = &state.you.id;
     let my_length = state.you.length;

     for snake in &state.board.snakes {
        if snake.id == *my_id || snake.length == 0 { continue; }

        let their_head = &snake.head;
        for &direction in &[Move::Up, Move::Down, Move::Left, Move::Right] {
            let their_potential_target = their_head.apply_move(direction);
            if !state.board.in_bounds(&their_potential_target) {
                continue;
            }
            // If opponent could move to our target and we are STRICTLY longer
            if their_potential_target == *target && my_length > snake.length {
                 debug!(
                    "Game {} Turn {}: Potential ADVANTAGEOUS head-to-head at ({},{}) with snake {} (length {} vs our {}). Considering.",
                    state.game.id, state.turn, target.x, target.y, snake.id, snake.length, my_length
                );
                return true;
            }
        }
     }
     false
} 