use crate::game_state::{Coord, GameState, Move};
use crate::sim::state::SimState;
use std::collections::{HashSet, VecDeque};

// Evaluates a list of safe moves based on the amount of space reachable
// from the resulting position.
// Returns a vector of (Move, space_count) tuples, sorted descending by space_count.
pub fn evaluate_moves_by_space(game_state: &GameState, safe_moves: &[Move]) -> Vec<(Move, usize)> {
    let mut move_scores = Vec::new();
    let head = &game_state.you.head;

    for &m in safe_moves {
        let next_pos = head.apply_move(m);
        // Check bounds again just in case, though safe_moves should guarantee it
        if !game_state.board.in_bounds(&next_pos) {
            continue;
        }
        let space = flood_fill(game_state, &next_pos);
        move_scores.push((m, space));
    }

    // Sort by available space (descending)
    move_scores.sort_by(|a, b| b.1.cmp(&a.1));
    move_scores
}

// Performs a flood fill starting from `start` to count accessible empty squares in a GameState.
// Kept for compatibility if needed elsewhere, but evaluation should use flood_fill_sim.
pub fn flood_fill(game_state: &GameState, start: &Coord) -> usize {
    let board = &game_state.board;
    let mut visited: HashSet<Coord> = HashSet::new();
    let mut queue: VecDeque<Coord> = VecDeque::new();

    // Create a set of all occupied points (all snake bodies, including self)
    // For flood fill, we treat our own body as an obstacle too.
    // Exclude tails as they will move out of the way.
    let occupied: HashSet<Coord> = board.snakes.iter().flat_map(|snake| {
        snake.body.iter().enumerate()
            .filter(|(i, _)| *i < snake.body.len() - 1) // Exclude the very last segment (tail tip)
            .map(|(_, segment)| *segment)
    }).collect();

    // Check if the start node itself is valid
    if !board.in_bounds(start) || occupied.contains(start) {
        return 0; // Cannot start fill from an invalid or occupied square
    }

    queue.push_back(*start);
    visited.insert(*start);

    while let Some(p) = queue.pop_front() {
        // Check all four adjacent cells
        for neighbor in p.neighbours() {
            // Skip if out of bounds
            if !board.in_bounds(&neighbor) {
                continue;
            }

            // Skip if already visited
            if visited.contains(&neighbor) {
                continue;
            }

            // Skip if occupied by a snake body (excluding tails)
            if occupied.contains(&neighbor) {
                continue;
            }

            // Skip hazards (optional, could add later)
            // if board.hazards.contains(&neighbor) {
            //     continue;
            // }

            // Mark as visited and add to queue
            visited.insert(neighbor);
            queue.push_back(neighbor);
        }
    }

    // Return the number of accessible cells (size of the visited set)
    visited.len()
}

// Performs a flood fill starting from `start` to count accessible empty squares in a SimState.
pub fn flood_fill_sim(sim_state: &SimState, start: &Coord) -> usize {
    let mut visited: HashSet<Coord> = HashSet::new();
    let mut queue: VecDeque<Coord> = VecDeque::new();

    // Create a set of occupied points from SimSnakes (excluding tails)
    let occupied: HashSet<Coord> = sim_state.snakes.iter().flat_map(|snake| {
        snake.body.iter().skip(1) // Skip head, only block body segments (tail is implicitly not included as we pop it)
            // Corrected logic: Exclude the *last* element (tail tip), not skip(1)
            // .enumerate()
            // .filter(|(i, _)| *i < snake.body.len() - 1) // Old logic, might be wrong for VecDeque
            // .map(|(_, segment)| *segment)

            // Let's iterate excluding the tail if it exists
            .take(if snake.body.len() > 1 { snake.body.len() - 1 } else { 0 })

    }).cloned().collect();


    // Check if the start node itself is valid
    if !sim_state.in_bounds(start) || occupied.contains(start) {
        return 0; // Cannot start fill from an invalid or occupied square
    }

    queue.push_back(*start);
    visited.insert(*start);

    while let Some(p) = queue.pop_front() {
        // Check all four adjacent cells
        for neighbor in p.neighbours() {
            // Skip if out of bounds
            if !sim_state.in_bounds(&neighbor) {
                continue;
            }

            // Skip if already visited
            if visited.contains(&neighbor) {
                continue;
            }

            // Skip if occupied by a snake body (excluding tails)
            if occupied.contains(&neighbor) {
                continue;
            }

            // Skip hazards (if added to SimState later)
            // if sim_state.hazards.contains(&neighbor) { continue; }

            // Mark as visited and add to queue
            visited.insert(neighbor);
            queue.push_back(neighbor);
        }
    }

    // Return the number of accessible cells (size of the visited set)
    visited.len()
} 