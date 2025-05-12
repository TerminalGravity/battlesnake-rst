use crate::game_state::{Coord, GameState, Move};
use super::head_to_head; // Import the head_to_head logic

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

// Basic fallback if no safe moves are found
pub fn fallback_safe_move() -> Move {
    Move::Down
} 