use crate::game_state::{GameState, Move};
use log::{debug, info, warn};

pub mod flood_fill;
pub mod safe_move;
pub mod food;
pub mod head_to_head;
pub mod evaluation;
pub mod search;

// Main function to decide the next move.
// It first gets all safe moves, then uses flood_fill to evaluate them.
// Falls back to the first safe move or a default if flood_fill yields no preference or an error.
pub fn decide_move(game_state: &GameState) -> Result<Move, String> {
    // Safe moves now includes L3 head-to-head avoidance
    let safe_moves = safe_move::get_safe_moves(game_state);
    debug!(
        "Game {} Turn {}: Safe moves (L0-L3): {:?}",
        game_state.game.id, game_state.turn, safe_moves
    );

    if safe_moves.is_empty() {
        warn!(
            "Game {} Turn {}: No safe moves found! Falling back.",
            game_state.game.id, game_state.turn
        );
        return Err("No safe moves available".to_string());
    }

    // TODO: Add config struct

    // --- Heuristic Layers (Priority Order) ---

    // TODO: Add config for search depth and enabling search
    let enable_search = false; // Disabled until implemented
    let search_depth = 3;

    // 1. L4: Minimax Search (if enabled)
    if enable_search {
        if let Some(search_move) = search::minimax_search(game_state, search_depth) {
            info!(
                "Game {} Turn {}: Chose move {} from L4 search.",
                game_state.game.id, game_state.turn, search_move.as_str()
            );
            return Ok(search_move);
        } else {
            debug!(
                "Game {} Turn {}: L4 Search did not return a move. Proceeding to L2.",
                game_state.game.id, game_state.turn
            );
        }
    }

    // 2. L2: Health & Food Management
    if food::should_seek_food(game_state) {
        if let Some(food_move) = food::find_move_to_closest_food(game_state, &safe_moves) {
            info!(
                "Game {} Turn {}: Health low ({}), using L2 Food logic: {}.",
                game_state.game.id, game_state.turn, game_state.you.health, food_move.as_str()
            );
            return Ok(food_move);
        } else {
            debug!(
                "Game {} Turn {}: Wanted food (L2), but no path. Proceeding to L1.",
                game_state.game.id, game_state.turn
            );
        }
    }

    // 3. L1: Flood Fill Space Heuristic
    // TODO: Add config to enable/disable
    let enable_flood_fill = true;
    if enable_flood_fill {
        let scored_moves = flood_fill::evaluate_moves_by_space(game_state, &safe_moves);
        debug!(
            "Game {} Turn {}: Scored moves (L1 flood_fill): {:?}",
            game_state.game.id, game_state.turn, scored_moves
        );

        if let Some((best_move, _score)) = scored_moves.first() {
             info!(
                "Game {} Turn {}: Using L1 Flood Fill logic: {}.",
                game_state.game.id, game_state.turn, best_move.as_str()
            );
            return Ok(*best_move);
        } else {
            debug!(
                "Game {} Turn {}: L1 Flood fill returned no moves. Falling back.",
                game_state.game.id, game_state.turn
            );
        }
    }

    // 4. Fallback: First available safe move (L0 safety ensures this is valid)
    let fallback_move = safe_moves.first().cloned().ok_or_else(|| {
        // This should be impossible if safe_moves wasn't empty at the start.
        "Critical error: No safe moves available and no fallback determined.".to_string()
    })?;
    warn!(
        "Game {} Turn {}: Falling back to first available safe move (L0): {}.",
        game_state.game.id, game_state.turn, fallback_move.as_str()
    );
    Ok(fallback_move)
} 