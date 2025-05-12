use crate::game_state::{Coord, GameState, Move, Battlesnake as ApiBattlesnake};
use std::collections::VecDeque; // Efficient for body manipulation

/// Lightweight representation of a snake for simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimSnake {
    pub id: String,       // Keep ID for tracking
    pub health: u32,
    pub body: VecDeque<Coord>, // Use VecDeque for efficient head/tail operations
                              // Length is implicitly body.len()
}

impl SimSnake {
    pub fn head(&self) -> Option<&Coord> {
        self.body.front()
    }

    pub fn length(&self) -> usize {
        self.body.len()
    }
}

/// Lightweight representation of the game state for simulation.
#[derive(Debug, Clone)]
pub struct SimState {
    pub width: i32,
    pub height: i32,
    pub snakes: Vec<SimSnake>,
    pub food: Vec<Coord>,      // Consider HashSet<Coord> if food checks are frequent
    pub turn: u32,             // Keep track for debugging/context
    // TODO: Add hazards if needed by ruleset
}

impl SimState {
    /// Placeholder for converting the full GameState from the API
    /// into a lightweight SimState for the search algorithm.
    pub fn from_api_state(api_state: &GameState) -> Self {
        SimState {
            width: api_state.board.width,
            height: api_state.board.height,
            snakes: api_state.board.snakes.iter().map(|api_snake| {
                SimSnake {
                    id: api_snake.id.clone(),
                    health: api_snake.health,
                    body: api_snake.body.iter().cloned().collect(), // Convert Vec to VecDeque
                }
            }).collect(),
            food: api_state.board.food.clone(),
            turn: api_state.turn,
        }
    }

    // TODO: Implement apply_moves(&mut self, moves: &[(String, Move)]) -> Self
    // This function will advance the simulation by one turn based on the chosen moves for each snake.
    // It needs to handle:
    // 1. Moving snake heads
    // 2. Checking for food consumption & health updates
    // 3. Removing snake tails (if no food eaten)
    // 4. Checking for collisions (wall, self, other)
    // 5. Removing dead snakes

     // Helper to check if a coordinate is within bounds
    pub fn in_bounds(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.x < self.width && coord.y >= 0 && coord.y < self.height
    }

    // TODO: Add helper methods as needed for collision detection, etc.
    // pub fn is_occupied(&self, coord: &Coord) -> bool { ... }

} 