use crate::game_state::{Coord, GameState, Move};
use crate::sim::state::{SimSnake, SimState};
use super::head_to_head; // Import the head_to_head logic
use std::collections::HashSet; // Added HashSet import

pub fn get_safe_moves(state: &GameState) -> Vec<Move> {
    let my_head = &state.you.head;
    let board = &state.board;
    let self_id = &state.you.id;

    let possible_moves = [Move::Up, Move::Down, Move::Left, Move::Right];
    let mut safe_moves = Vec::new();

    for &direction in &possible_moves {
        let target = my_head.apply_move(direction);

        // 1. Wall collision check
        if !board.in_bounds(&target) {
            continue;
        }

        // 2. Self collision check
        if board.is_occupied_by_snake(&target, self_id) {
             let tail_tip = state.you.body.last().unwrap_or(my_head);
             if target == *tail_tip && state.you.length > 1 && target != state.you.head {
                // Allow moving onto tail tip for now (can be refined)
             } else {
                continue; // Avoid self collision
             }
        }

        // 3. Other snake body collision check (excluding tails and heads for now)
        if board.is_occupied_by_others(&target, self_id, true) {
            // Need to refine this: board.is_occupied_by_others excludes tails.
            // We need to specifically check if the target is another snake's head.
            let mut occupied_by_other_body_not_head = false;
            for other_snake in state.board.snakes.iter().filter(|s| s.id != *self_id) {
                // Check body segments *except* the head (index 0)
                for segment in other_snake.body.iter().skip(1) {
                    if *segment == target {
                        occupied_by_other_body_not_head = true;
                        break;
                    }
                }
                if occupied_by_other_body_not_head { break; }
            }
            if occupied_by_other_body_not_head {
                 continue; // Avoid non-head body parts
            }
        }

        // 4. Head-to-head collision check (L3)
        if head_to_head::is_dangerous_head_to_head(state, &target) {
            continue; // Avoid dangerous head-to-head
        }

        // 5. Hazard check (Optional)
        // if board.hazards.contains(&target) { continue; }

        safe_moves.push(direction);
    }

    // Optional: If advantageous head-to-head moves exist, prioritize them?
    // For now, just return all moves deemed safe by the above checks.

    safe_moves
}

// Calculates safe moves for a specific snake within a SimState.
pub fn get_sim_safe_moves(state: &SimState, snake_id: &str) -> Vec<Move> {
    let snake = match state.snakes.iter().find(|s| s.id == snake_id) {
        Some(s) => s,
        None => return vec![], // Snake not found or already dead
    };
    let head = match snake.head() {
        Some(h) => h,
        None => return vec![], // Snake has no head (empty body)
    };

    let possible_moves = [Move::Up, Move::Down, Move::Left, Move::Right];
    let mut safe_moves = Vec::new();

    // Precompute occupied set for efficiency
    let occupied_coords: HashSet<Coord> = state.snakes.iter()
        .flat_map(|s| s.body.iter().cloned())
        .collect();
    
    // Precompute occupied excluding tails (for other snake check)
     let occupied_excluding_tails: HashSet<Coord> = state.snakes.iter().flat_map(|s| {
        // Iterate over body segments, excluding the tail tip if len > 1
        s.body.iter().take(if s.body.len() > 1 { s.body.len() - 1 } else { s.body.len() })
    }).cloned().collect();

    for &direction in &possible_moves {
        let target = head.apply_move(direction);

        // 1. Wall collision check
        if !state.in_bounds(&target) {
            continue;
        }

        // 2. Body collision check (any snake, including self, excluding tails)
        // Check against occupied_excluding_tails set
        if occupied_excluding_tails.contains(&target) {
             // Is the target *this* snake's tail tip? (Allow moving onto own tail)
             if let Some(tail_tip) = snake.body.back() {
                 if target == *tail_tip && snake.length() > 1 {
                     // It's our own tail tip, potentially safe.
                     // Need to ensure it's not *also* another snake's non-tail body part.
                      let occupied_by_other_non_tail = state.snakes.iter()
                            .filter(|s| s.id != snake_id)
                            .flat_map(|s| s.body.iter().take(if s.body.len() > 1 { s.body.len() - 1 } else { s.body.len() }))
                            .any(|seg| *seg == target);
                     if occupied_by_other_non_tail {
                         continue; // Tail occupied by other snake body
                     }
                     // Otherwise, allow moving onto own tail
                 } else {
                     continue; // Occupied by non-tail body part
                 }
             } else {
                 continue; // Occupied by non-tail body part (or snake has no tail?)
             }
        }
        
        // 3. Head-to-head collision check (basic version for sim)
        // TODO: Refactor head_to_head logic to potentially work with SimState directly
        // For now, skip this check in the simulation's safe move calculation 
        // The main evaluation will handle head-to-head implicitly via outcomes.
        // let temp_gamestate = state.to_api_state(); // Requires conversion function
        // if head_to_head::is_dangerous_head_to_head(&temp_gamestate, &target) {
        //     continue;
        // }

        // 4. Hazard check (if hazards are added to SimState)

        safe_moves.push(direction);
    }
    safe_moves
}

// Basic fallback if no safe moves are found
pub fn fallback_safe_move() -> Move {
    Move::Down
} 