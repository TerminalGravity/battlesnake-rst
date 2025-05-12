use crate::sim::state::{SimState, SimSnake}; // Use SimState
use super::flood_fill; // Use flood_fill module
use log::debug;

// Calculate controlled space for a specific snake in a SimState
fn calculate_controlled_space(state: &SimState, snake: &SimSnake) -> usize {
    match snake.head() {
        Some(head) => flood_fill::flood_fill_sim(state, head),
        None => 0, // No space if snake has no head (is dead)
    }
}

// Evaluates a simulated game state from the perspective of the specified snake ID.
// Higher scores are better.
// TODO: Refine weights, add more factors (food proximity, opponent threats, center control, etc.)
pub fn evaluate_sim_state(state: &SimState, our_id: &str) -> i32 {

     let you = match state.snakes.iter().find(|s| s.id == our_id) {
         Some(s) => s,
         None => return i32::MIN + 1, // We are dead (use MIN + 1 to distinguish from deeper losses)
     };

    // Check for win (only snake left)
    if state.snakes.len() == 1 {
        return i32::MAX - 1; // We won (use MAX - 1 to allow depth preference)
    }

    // --- Component Scores --- 
    let health_score = you.health as i32; // Weight: 1
    let length_score = you.length() as i32 * 10; // Weight: 10 (length is crucial)
    
    // Space Score - Use flood fill from our head in the SimState
    let space_score = calculate_controlled_space(state, you) as i32 * 2; // Weight: 2

    // Length Advantage Score - Compare our length to the *longest* opponent
    let max_opponent_length = state.snakes.iter()
        .filter(|s| s.id != our_id)
        .map(|s| s.length())
        .max()
        .unwrap_or(0); // If no opponents, advantage is based on 0 length
    
    let length_advantage = (you.length() as i32 - max_opponent_length as i32) * 5; // Weight: 5

    // --- Aggregation --- 
    let score = health_score 
                + length_score 
                + space_score 
                + length_advantage;
    
     debug!(
        "Game Turn {}: Eval for {}: Score={}, (H={}, L={}, S={}, LA={})",
        state.turn, our_id, score, health_score, length_score, space_score, length_advantage
    );
    
    score
}

// Remove old GameState-based evaluation functions
/*
pub fn evaluate_state(state: &GameState) -> i32 { ... }
pub fn evaluate_state_v2(state: &GameState) -> i32 { ... }
*/ 