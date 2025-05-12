use crate::game_state::{GameState};
use super::flood_fill; // Use existing flood_fill for space evaluation
use log::debug;

// Basic heuristic score for a given game state from the perspective of 'you'.
// Higher scores are better.
// TODO: Refine weights and add more factors (food proximity, opponent threats, etc.)
pub fn evaluate_state(state: &GameState) -> i32 {
    let my_id = &state.you.id;

    // Check for immediate death
    if !state.board.snakes.iter().any(|s| s.id == *my_id) {
        return i32::MIN; // Lost state
    }

    // Check for win (only snake left)
    if state.board.snakes.len() == 1 && state.board.snakes[0].id == *my_id {
        return i32::MAX; // Won state
    }

    let health_score = state.you.health as i32;
    let length_score = state.you.length as i32 * 10; // Length is important

    // Calculate controlled space using flood fill from our head
    // Use a large max value for fill, assuming we want full area in evaluation
    let space_score = flood_fill::evaluate_moves_by_space(state, &[/*Need moves here, maybe evaluate space directly?*/])
        .first()
        .map_or(0, |(_, score)| *score as i32);

    // Simple aggregation for now
    let score = health_score + length_score + space_score;
    debug!(
        "Game {} Turn {}: Evaluated state score: {} (H: {}, L: {}, S: {})",
        state.game.id, state.turn, score, health_score, length_score, space_score
    );
    score
}

// Need to rethink how to get space score without moves. Maybe run flood fill directly?
fn calculate_controlled_space(state: &GameState) -> usize {
    flood_fill::flood_fill(state, &state.you.head)
}

// Refined evaluation function incorporating space directly
pub fn evaluate_state_v2(state: &GameState) -> i32 {
     let my_id = &state.you.id;
     let you = match state.board.snakes.iter().find(|s| s.id == *my_id) {
         Some(s) => s,
         None => return i32::MIN, // We are dead
     };

    if state.board.snakes.len() == 1 {
        return i32::MAX; // We won
    }

    let health_score = you.health as i32;
    let length_score = you.length as i32 * 10;
    let space_score = calculate_controlled_space(state) as i32;

     // Add differential components?
    let mut length_advantage = 0;
    for snake in &state.board.snakes {
        if snake.id != *my_id {
            length_advantage += (you.length as i32 - snake.length as i32);
        }
    }

    let score = health_score + length_score + space_score + length_advantage;
     debug!(
        "Game {} Turn {}: Evaluated state score v2: {} (H: {}, L: {}, S: {}, LA: {})",
        state.game.id, state.turn, score, health_score, length_score, space_score, length_advantage
    );
    score
} 