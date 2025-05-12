use crate::game_state::{Coord, GameState, Move, Battlesnake as ApiBattlesnake};
use std::collections::{VecDeque, HashMap, HashSet}; // Added HashMap, HashSet

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

    // Helper to check if a coord is part of this snake's body
    fn occupies(&self, coord: &Coord) -> bool {
        self.body.contains(coord)
    }
}

/// Lightweight representation of the game state for simulation.
#[derive(Debug, Clone)]
pub struct SimState {
    pub width: i32,
    pub height: i32,
    pub snakes: Vec<SimSnake>,
    pub food: HashSet<Coord>, // Use HashSet for faster food lookups
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
            food: api_state.board.food.iter().cloned().collect(), // Convert Vec to HashSet
            turn: api_state.turn,
        }
    }

    /// Simulates one turn of the game based on the provided moves.
    /// `moves`: A map where key is snake ID and value is the chosen Move.
    pub fn apply_moves(&self, moves: &HashMap<String, Move>) -> Self {
        let mut next_state = self.clone();
        next_state.turn += 1;

        let mut ate_food: HashSet<String> = HashSet::new();
        let mut next_head_positions: HashMap<String, Coord> = HashMap::new();

        // 1. Calculate next head positions and decrease health
        for snake in &mut next_state.snakes {
            snake.health = snake.health.saturating_sub(1); // Decrease health
            let current_head = match snake.head() {
                Some(h) => h,
                None => continue, // Snake already effectively dead (empty body)
            };
            // Use provided move or default to a non-moving state (e.g., current head)
            // In a real search, we'd likely have moves for all snakes or prune dead branches
            let chosen_move = moves.get(&snake.id).copied().unwrap_or_else(|| {
                // Default behavior if no move provided (e.g., assume 'up' or stay still?)
                // For simulation, maybe assume it continues straight or uses a simple heuristic?
                // Let's assume 'up' for now as a placeholder default if a snake's move is missing.
                Move::Up
            });
            next_head_positions.insert(snake.id.clone(), current_head.apply_move(chosen_move));
        }

        // 2. Food Consumption
        let mut food_to_remove: HashSet<Coord> = HashSet::new();
        for snake in &mut next_state.snakes {
            if let Some(next_head) = next_head_positions.get(&snake.id) {
                if next_state.food.contains(next_head) {
                    ate_food.insert(snake.id.clone());
                    snake.health = 100;
                    food_to_remove.insert(*next_head);
                }
            }
        }
        next_state.food = next_state.food.difference(&food_to_remove).cloned().collect();

        // 3. Move snake bodies (Grow or Shrink)
        for snake in &mut next_state.snakes {
             if let Some(next_head) = next_head_positions.get(&snake.id) {
                 snake.body.push_front(*next_head); // Add new head
                 if !ate_food.contains(&snake.id) {
                     snake.body.pop_back(); // Remove tail if no food eaten
                 }
             }
        }

        // 4. Collision Detection - Determine who dies
        let mut died_this_turn: HashSet<String> = HashSet::new();
        let mut occupied_squares: HashMap<Coord, Vec<String>> = HashMap::new();

        // Check wall, health, and self-collision
        for snake in &next_state.snakes {
             if snake.health == 0 {
                 died_this_turn.insert(snake.id.clone());
                 continue;
             }
             if let Some(head) = snake.head() {
                 if !next_state.in_bounds(head) {
                     died_this_turn.insert(snake.id.clone());
                     continue;
                 }
                 // Check self-collision (new head vs rest of new body)
                 if snake.body.iter().skip(1).any(|segment| segment == head) {
                     died_this_turn.insert(snake.id.clone());
                     continue;
                 }
                 // Track occupied squares for inter-snake collision checks
                 occupied_squares.entry(*head).or_default().push(snake.id.clone());
             } else {
                 died_this_turn.insert(snake.id.clone()); // Died if body is empty
             }
        }

        // Check inter-snake collisions (body and head-to-head)
        for snake in &next_state.snakes {
            if died_this_turn.contains(&snake.id) { continue; }
            if let Some(head) = snake.head() {
                // Check against other snakes' bodies (excluding their heads)
                for other_snake in &next_state.snakes {
                    if snake.id == other_snake.id || died_this_turn.contains(&other_snake.id) { continue; }
                    if other_snake.body.iter().skip(1).any(|segment| segment == head) {
                        died_this_turn.insert(snake.id.clone());
                        break;
                    }
                }
                if died_this_turn.contains(&snake.id) { continue; }

                // Check head-to-head using occupied_squares map
                if let Some(colliders) = occupied_squares.get(head) {
                    if colliders.len() > 1 { // Head-to-head collision occurred
                        let max_len = colliders.iter()
                            .filter_map(|id| next_state.snakes.iter().find(|s| &s.id == id))
                            .map(|s| s.length())
                            .max()
                            .unwrap_or(0);

                        // Anyone not matching max length dies
                        for id in colliders {
                             if let Some(s) = next_state.snakes.iter().find(|s| &s.id == id) {
                                 if s.length() < max_len {
                                     died_this_turn.insert(id.clone());
                                 }
                             }
                        }
                        // If multiple snakes have the same max length, they all die
                         let max_len_colliders: Vec<_> = colliders.iter()
                             .filter_map(|id| next_state.snakes.iter().find(|s| &s.id == id))
                             .filter(|s| s.length() == max_len)
                             .collect();
                        if max_len_colliders.len() > 1 {
                            for s in max_len_colliders {
                                died_this_turn.insert(s.id.clone());
                            }
                        }
                    }
                }
            }
        }

        // 5. Remove dead snakes
        next_state.snakes.retain(|snake| !died_this_turn.contains(&snake.id));

        next_state
    }

     // Helper to check if a coordinate is within bounds
    pub fn in_bounds(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.x < self.width && coord.y >= 0 && coord.y < self.height
    }

    // TODO: Add helper methods as needed for collision detection, etc.
    // pub fn is_occupied(&self, coord: &Coord) -> bool { ... }

} 