use crate::game_state::{GameState, Move};
use log::debug;

pub mod flood_fill;
pub mod safe_move;

// Main function to decide the next move.
// It first gets all safe moves, then uses flood_fill to evaluate them.
// Falls back to the first safe move or a default if flood_fill yields no preference or an error.
pub fn decide_move(game_state: &GameState) -> Result<Move, String> {
    let safe_moves = safe_move::get_safe_moves(game_state);
    debug!("Game {} Turn {}: Safe moves: {:?}", game_state.game.id, game_state.turn, safe_moves);

    if safe_moves.is_empty() {
        debug!("Game {} Turn {}: No safe moves found. Using fallback.", game_state.game.id, game_state.turn);
        // Return an error or a default move. For now, let main.rs handle the fallback.
        return Err("No safe moves available".to_string());
    }

    // TODO: Add config check here for feature_flags.enable_flood_fill
    let enable_flood_fill = true; // Assuming enabled for now

    if enable_flood_fill {
        let scored_moves = flood_fill::evaluate_moves_by_space(game_state, &safe_moves);
        debug!("Game {} Turn {}: Scored moves (flood_fill): {:?}", game_state.game.id, game_state.turn, scored_moves);

        if let Some((best_move, _score)) = scored_moves.first() {
            return Ok(*best_move);
        } else {
            debug!("Game {} Turn {}: Flood fill returned no moves. Falling back to first safe move.", game_state.game.id, game_state.turn);
            // Fallback to the first safe move if flood fill evaluation is empty for some reason
            // (should not happen if safe_moves is not empty and flood_fill is correct)
            if let Some(first_safe) = safe_moves.first() {
                return Ok(*first_safe);
            } else {
                 // This case should be practically impossible if safe_moves was not empty initially
                return Err("No moves available after flood fill and no fallback safe move.".to_string());
            }
        }
    }

    // Default fallback if flood fill is not enabled or fails to provide a move
    debug!("Game {} Turn {}: Flood fill not enabled or failed. Using first safe move.", game_state.game.id, game_state.turn);
    Ok(safe_moves.first().cloned().unwrap_or_else(|| {
        // This part should ideally be unreachable if safe_moves had elements.
        // If safe_moves was empty, we already returned Err.
        debug!("Game {} Turn {}: Critical fallback: No safe moves, returning default. This indicates an issue.", game_state.game.id, game_state.turn);
        safe_move::fallback_safe_move()
    }))
} 