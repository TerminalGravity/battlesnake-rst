use crate::game_state::{GameState, Move};
use crate::sim::state::{SimState, SimSnake};
use crate::logic::safe_move::get_sim_safe_moves;
use crate::logic::flood_fill::flood_fill_sim;
use super::evaluation;
use log::{debug, warn, info};
use std::collections::HashMap;
use std::time::{Duration, Instant};

const MAX_SEARCH_TIME_MS: u128 = 400; // Max time before fallback (adjust as needed)

// --- Top-level Search Function ---

// Finds the best move using minimax search within a time limit.
pub fn minimax_search(state: &GameState, depth: u8) -> Option<Move> {
    let overall_start_time = Instant::now();
    info!(
        "Game {} Turn {}: === Starting Minimax search (depth {}) ===",
        state.game.id, state.turn, depth
    );
    let sim_state_initial = SimState::from_api_state(state);
    let our_id = &sim_state_initial.snakes.iter().find(|s| s.id == state.you.id)?.id.clone(); // Find our ID in sim state

    let legal_moves = get_sim_safe_moves(&sim_state_initial, our_id);
    if legal_moves.is_empty() {
        warn!("Minimax Search: No legal moves found initially!");
        return None;
    }
    if legal_moves.len() == 1 {
        debug!("Minimax Search: Only one legal move, returning early.");
        return Some(legal_moves[0]);
    }

    let mut best_move = *legal_moves.first().unwrap_or(&Move::Down); // Default to first safe or down
    let mut best_score = i32::MIN;

    // Iterate through our first set of moves
    for &move_option in &legal_moves {
        let move_start_time = Instant::now();
        let next_sim_state = simulate_turn_with_heuristic_opponents(&sim_state_initial, our_id, move_option);
        
        let score = minimax(
            next_sim_state,
            depth - 1,
            i32::MIN,
            i32::MAX,
            false, // Opponent's turn next
            our_id,
            overall_start_time, // Pass overall start time for timeout check
        );

        let move_duration = move_start_time.elapsed();
        debug!("  -> Eval Move: {:?}, Score: {}, Time: {:?}", move_option, score, move_duration);
        if score > best_score {
            best_score = score;
            best_move = move_option;
        }
         // Check overall time limit 
        if overall_start_time.elapsed().as_millis() > MAX_SEARCH_TIME_MS {
            warn!("Minimax search TIMED OUT after {:?}! Returning best move found so far: {:?}", overall_start_time.elapsed(), best_move);
            return Some(best_move);
        }
    }
    let total_duration = overall_start_time.elapsed();
    info!("=== Minimax Search END. Best Move: {:?}, Score: {}, Total Time: {:?} ===", best_move, best_score, total_duration);
    Some(best_move)
}

// --- Minimax Recursive Helper ---
fn minimax(
    state: SimState,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    is_maximizing_player: bool,
    our_id: &str,
    start_time: Instant,
) -> i32 {
    // Check time limit first
    let elapsed = start_time.elapsed();
    if elapsed.as_millis() > MAX_SEARCH_TIME_MS {
        warn!("Timeout hit inside minimax recursion at depth {}. Returning eval.", depth);
        return evaluation::evaluate_sim_state(&state, our_id); 
    }
    
    // Base Case: Leaf node (depth 0 or terminal state)
    if depth == 0 || state.snakes.len() <= 1 || state.snakes.iter().all(|s| s.health == 0) {
        return evaluation::evaluate_sim_state(&state, our_id);
    }
    let current_snake_turn_id = if is_maximizing_player { our_id.to_string() } else { 
        // Simplification: Assume minimizer controls the *next* opponent snake in the list? 
        // Or just evaluate based on the state after *all* opponents move heuristically?
        // Let's stick with the latter for now.
        // We need the state *after* opponents make their move below.
         state.snakes.iter().find(|s| s.id != our_id).map(|s| s.id.clone()).unwrap_or_default()
    };
    if current_snake_turn_id.is_empty() && !is_maximizing_player { // Only our snake left? 
         return evaluation::evaluate_sim_state(&state, our_id); // Should be caught by snakes.len() <= 1, but safe check.
    }

    if is_maximizing_player {
        // Our turn (Maximizing)
        let mut max_eval = i32::MIN;
        let legal_moves = get_sim_safe_moves(&state, our_id);
        if legal_moves.is_empty() {
            return evaluation::evaluate_sim_state(&state, our_id); // Evaluate state if we have no moves
        }

        for &move_option in &legal_moves {
            let next_sim_state = simulate_turn_with_heuristic_opponents(&state, our_id, move_option);
            let eval = minimax(next_sim_state, depth - 1, alpha, beta, false, our_id, start_time);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break; // Beta cutoff
            }
        }
        max_eval
    } else {
        // Opponent's turn (Minimizing) - Assume they play heuristically
        // Note: This isn't true minimax, but a heuristic search.
        // The state passed here *should* be the result of our previous move.
        // We now simulate the opponents playing their *best* heuristic move.
        let opponent_moves = predict_opponent_moves_heuristic(&state, our_id);
        let next_state_after_opponents = state.apply_moves(&opponent_moves); 

        minimax(next_state_after_opponents, depth - 1, alpha, beta, true, our_id, start_time)
        
        // --- True Minimax (More Complex) requires iterating opponent moves --- 
        /*
        let mut min_eval = i32::MAX;
        // Need opponent move generation here
        let opponent_ids: Vec<_> = state.snakes.iter().filter(|s| s.id != our_id).map(|s| s.id.clone()).collect();
        if opponent_ids.is_empty() {
             return evaluation::evaluate_sim_state(&state, our_id);
        }
        // Simplified: iterate only the *first* opponent's moves for pruning estimate
        let first_opponent_id = opponent_ids[0].clone();
        let opponent_legal_moves = get_sim_safe_moves(&state, &first_opponent_id);
        if opponent_legal_moves.is_empty() {
             return evaluation::evaluate_sim_state(&state, our_id); // Opponent has no moves
        }
        
        for &opp_move in &opponent_legal_moves {
            let mut moves_for_turn = HashMap::new();
            moves_for_turn.insert(first_opponent_id.clone(), opp_move);
            // Add heuristic moves for other opponents?
            // ... 
            let next_sim_state = state.apply_moves(&moves_for_turn);
            let eval = minimax(next_sim_state, depth - 1, alpha, beta, true, our_id, start_time);
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break; // Alpha cutoff
            }
        }
        min_eval
        */
    }
}

// --- Opponent Move Prediction Helper ---

// Simple heuristic: Opponents choose their move maximizing their own flood fill space.
fn predict_opponent_moves_heuristic(state: &SimState, our_id: &str) -> HashMap<String, Move> {
    let mut opponent_moves = HashMap::new();
    for snake in &state.snakes {
        if snake.id == our_id { continue; }

        let legal_moves = get_sim_safe_moves(state, &snake.id);
        if legal_moves.is_empty() {
            // If an opponent has no safe moves, they effectively make no move (and likely die)
            // We could represent this differently, but for apply_moves, skipping their move entry works.
             continue; 
        }

        let mut best_opp_move = *legal_moves.first().unwrap_or(&Move::Up); // Default
        let mut best_opp_score = 0; // Flood fill space

        for &opp_move in &legal_moves {
            if let Some(head) = snake.head() {
                 let target = head.apply_move(opp_move);
                 // Evaluate based on flood fill from the target square
                 let space = flood_fill_sim(state, &target);
                 if space > best_opp_score {
                     best_opp_score = space;
                     best_opp_move = opp_move;
                 }
            }
        }
        opponent_moves.insert(snake.id.clone(), best_opp_move);
    }
    opponent_moves
}

// Helper to simulate a full turn given our move and predicting opponents' moves heuristically
fn simulate_turn_with_heuristic_opponents(state: &SimState, our_id: &str, our_move: Move) -> SimState {
    let mut moves_for_turn = predict_opponent_moves_heuristic(state, our_id);
    moves_for_turn.insert(our_id.to_string(), our_move);
    state.apply_moves(&moves_for_turn)
} 