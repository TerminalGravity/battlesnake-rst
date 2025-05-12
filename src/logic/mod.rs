use crate::game_state::{GameState, Move};
use crate::logic::search::minimax_search; // Ensure minimax_search is importable
use log::{debug, info, warn};

pub mod flood_fill;
pub mod safe_move;
pub mod food;
pub mod head_to_head;
pub mod evaluation;
pub mod search;

// --- Constants for Ruleset Names (match API spec) ---
const RULESET_STANDARD: &str = "standard";
const RULESET_SOLO: &str = "solo";
const RULESET_ROYALE: &str = "royale";
const RULESET_SQUAD: &str = "squad";
const RULESET_CONSTRICTOR: &str = "constrictor";
const RULESET_WRAPPED: &str = "wrapped";

// Main function to decide the next move.
pub fn decide_move(game_state: &GameState) -> Result<Move, String> {
    let start_time = std::time::Instant::now();
    let game_id = &game_state.game.id;
    let turn = game_state.turn;
    let ruleset_name = &game_state.game.ruleset.name;
    info!(
        "Game {} Turn {} Ruleset: {} ----- Deciding Move ----- ",
        game_id, turn, ruleset_name
    );

    // TODO: Potentially adjust behavior based on ruleset early on (e.g., wrapped board logic)

    // L0-L3 Safe Moves (includes head-to-head avoidance)
    // TODO: Wrapped mode needs different boundary checks in safe_move and flood_fill
    let safe_moves = safe_move::get_safe_moves(game_state);
    debug!("[{:?}] Safe moves (L0-L3): {:?}", start_time.elapsed(), safe_moves);

    if safe_moves.is_empty() {
        warn!("[{:?}] No safe moves found!", start_time.elapsed());
        return Err("No safe moves available".to_string());
    }
    if safe_moves.len() == 1 {
        info!(
            "[{:?}] Only one safe move: {:?}. Choosing early.",
            start_time.elapsed(), safe_moves[0]
        );
        return Ok(safe_moves[0]);
    }

    // --- Heuristic Layers (Priority Order) ---

    // TODO: Read config
    let enable_search = true;
    let search_depth = 4;
    // TODO: Adjust search depth based on ruleset/remaining time?

    // 1. L4: Minimax Search
    if enable_search {
        // Pass ruleset name to search/evaluation if needed?
        let search_result = search::minimax_search(game_state, search_depth);
        if let Some(search_move) = search_result {
            info!(
                "[{:?}] Chose move {} via L4 Minimax Search.",
                start_time.elapsed(), search_move.as_str()
            );
            return Ok(search_move);
        } else {
            warn!(
                "[{:?}] L4 Search failed or timed out. Falling through.",
                start_time.elapsed()
            );
        }
    }

    // 2. L2: Health & Food Management
    // TODO: Adjust food seeking threshold/logic based on ruleset?
    // e.g., In Constrictor, food is less important than trapping.
    //       In Royale, food might be needed more aggressively late game.
    let seek_food = if ruleset_name == RULESET_CONSTRICTOR {
        false // In Constrictor, space control is usually paramount
    } else {
        food::should_seek_food(game_state)
    };

    if seek_food {
        debug!(
            "[{:?}] Checking L2 Food Logic (Health: {}).",
            start_time.elapsed(), game_state.you.health
        );
        if let Some(food_move) = food::find_move_to_closest_food(game_state, &safe_moves) {
            info!(
                "[{:?}] Chose move {} via L2 Food Logic.",
                start_time.elapsed(), food_move.as_str()
            );
            return Ok(food_move);
        } else {
            debug!("[{:?}] L2: No food path found.", start_time.elapsed());
        }
    }

    // 3. L1: Flood Fill Space Heuristic
    // TODO: Add hazard avoidance in flood_fill for rulesets like Royale?
    let enable_flood_fill = true;
    if enable_flood_fill {
        debug!("[{:?}] Checking L1 Flood Fill Logic.", start_time.elapsed());
        // Need to pass GameState here, not SimState if flood_fill needs hazard info not in SimState
        let scored_moves = flood_fill::evaluate_moves_by_space(game_state, &safe_moves);
        debug!("[{:?}] L1 Scored moves: {:?}", start_time.elapsed(), scored_moves);

        if let Some((best_move, _score)) = scored_moves.first() {
            // TODO: Add ruleset-specific tie-breaking?
            info!(
                "[{:?}] Chose move {} via L1 Flood Fill Logic.",
                start_time.elapsed(), best_move.as_str()
            );
            return Ok(*best_move);
        } else {
            debug!("[{:?}] L1 Flood fill returned no preference.", start_time.elapsed());
        }
    }

    // 4. Fallback: First available safe move (L0)
    let fallback_move = safe_moves.first().cloned().ok_or_else(|| {
        "Critical error: No safe moves available and no fallback determined.".to_string()
    })?;
    warn!(
        "[{:?}] No heuristic chose a move. Falling back to L0 (first safe): {}.",
        start_time.elapsed(), fallback_move.as_str()
    );
    Ok(fallback_move)
} 