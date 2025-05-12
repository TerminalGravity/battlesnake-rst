use crate::game_state::{Coord, GameState, Move};

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

        // 2. Self collision check (avoid all body parts, including future head position)
        // Careful: Naive check might prevent moving onto tail square even if safe.
        // For L0, simple avoidance is sufficient.
        if board.is_occupied_by_snake(&target, self_id) {
             // Allow moving onto tail tip only if length > 1 and it's not head
             let tail_tip = state.you.body.last().unwrap_or(my_head); // default to head if body is empty somehow
             if target == *tail_tip && state.you.length > 1 && target != state.you.head {
                 // It's the tail tip (and not also the head), potentially safe to move onto
                 // Advanced logic needed here. For basic safety, still avoid.
                 // continue;
             } else {
                continue; // Avoid self collision
             }
        }

        // 3. Other snake collision check
        // Avoid body parts. Head-to-head needs L3 logic.
        // Exclude tails for now, as they will move.
        if board.is_occupied_by_others(&target, self_id, true) {
            continue;
        }

        // 4. Hazard check (Optional, depending on ruleset - not implemented here)
        // if board.hazards.contains(&target) {
        //     continue;
        // }

        safe_moves.push(direction);
    }

    safe_moves
}

// Basic fallback if no safe moves are found (should ideally not happen)
pub fn fallback_safe_move() -> Move {
    Move::Down
} 