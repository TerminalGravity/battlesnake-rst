use crate::game_state::{GameState, Move};
use crate::logic::search::minimax_search;
use log::{debug, info, warn};
use std::env; // Added for environment variable access

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

    // Allow overriding ruleset via environment variable for testing
    let ruleset_name_from_engine = game_state.game.ruleset.name.as_str();
    let forced_ruleset = env::var("FORCE_RULESET").ok();
    let effective_ruleset_name = forced_ruleset.as_deref().unwrap_or(ruleset_name_from_engine);

    if let Some(ref forced) = forced_ruleset {
        info!(
            "Game {} Turn {}: Using FORCED Ruleset: '{}' (Engine sent: '{}')",
            game_id, turn, forced, ruleset_name_from_engine
        );
    } else {
        info!(
            "Game {} Turn {}: Ruleset: '{}' ----- Deciding Move ----- ",
            game_id, turn, effective_ruleset_name
        );
    }

    // --- Ruleset-Specific Adjustments (Early) ---
    let is_wrapped_mode = effective_ruleset_name == RULESET_WRAPPED;
    // TODO: Pass `is_wrapped_mode` to safe_move::get_safe_moves, sim::state::apply_moves, 
    //       flood_fill, and evaluation functions for boundary condition changes.

    // L0-L3 Safe Moves
    let safe_moves = safe_move::get_safe_moves(game_state); // This uses game_state, so it naturally gets engine ruleset
                                                          // If wrapped mode affects safe_moves, it needs the effective_ruleset_name or is_wrapped_mode.
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

    // --- Heuristic Layers & Ruleset Adjustments ---
    // TODO: Move config (depth, flags, weights) to a struct/env vars
    let mut enable_search = true;
    let mut search_depth = 4;
    let mut food_seek_health_threshold = food::DEFAULT_FOOD_THRESHOLD;
    let mut enable_flood_fill = true;

    match effective_ruleset_name {
        RULESET_STANDARD | RULESET_SOLO => {
            info!("[{:?}] Applying Standard/Solo ruleset logic.", start_time.elapsed());
            // Defaults are generally fine. No specific overrides needed here for standard.
        }
        RULESET_CONSTRICTOR => {
            info!("[{:?}] Applying Constrictor ruleset logic.", start_time.elapsed());
            food_seek_health_threshold = 15;
            // search_depth = 5; // Consider deeper search for trapping
        }
        RULESET_ROYALE => {
            info!("[{:?}] Applying Royale ruleset logic.", start_time.elapsed());
            food_seek_health_threshold = 60;
            // TODO: Modify SimState/evaluation/flood_fill to handle hazards.
            // TODO: safe_moves needs to check for hazards in Royale.
        }
        RULESET_WRAPPED => {
            info!("[{:?}] Applying Wrapped ruleset logic (boundary checks are TODO).", start_time.elapsed());
            // Primary change is boundary logic, passed via is_wrapped_mode where needed.
        }
        _ => {
            warn!("[{:?}] Unknown ruleset '{}', using default heuristics.", start_time.elapsed(), effective_ruleset_name);
        }
    }

    // 1. L4: Minimax Search
    if enable_search {
        // TODO: Pass effective_ruleset_name or derived config to search/evaluation 
        //       if their internal logic needs to adapt (e.g., different eval weights).
        let search_result = search::minimax_search(game_state, search_depth /*, &ruleset_config */);
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

    // 2. L2: Health & Food Management (using potentially adjusted threshold)
    if game_state.you.health < food_seek_health_threshold {
        debug!(
            "[{:?}] Checking L2 Food Logic (Health: {}, Threshold: {}).",
            start_time.elapsed(), game_state.you.health, food_seek_health_threshold
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
    if enable_flood_fill {
        debug!("[{:?}] Checking L1 Flood Fill Logic.", start_time.elapsed());
        // TODO: flood_fill::evaluate_moves_by_space needs to handle wrapped and hazards based on effective_ruleset_name
        let scored_moves = flood_fill::evaluate_moves_by_space(game_state, &safe_moves);
        debug!("[{:?}] L1 Scored moves: {:?}", start_time.elapsed(), scored_moves);

        if let Some((best_move, _score)) = scored_moves.first() {
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