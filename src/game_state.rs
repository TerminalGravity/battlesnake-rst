use serde::{Deserialize, Serialize};

// Represents the primary directions a Battlesnake can move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Move {
    Up,
    Down,
    Left,
    Right,
}

// Convert Move enum to the string expected by the Battlesnake API.
impl Move {
    pub fn as_str(&self) -> &'static str {
        match self {
            Move::Up => "up",
            Move::Down => "down",
            Move::Left => "left",
            Move::Right => "right",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)] // Added Copy, PartialEq, Eq, Hash
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

// Helper methods for Coord
impl Coord {
    // Returns the 4 neighboring coordinates
    pub fn neighbours(&self) -> [Coord; 4] {
        [
            Coord { x: self.x, y: self.y + 1 }, // Up
            Coord { x: self.x, y: self.y - 1 }, // Down
            Coord { x: self.x - 1, y: self.y }, // Left
            Coord { x: self.x + 1, y: self.y }, // Right
        ]
    }

    // Calculates the coordinate resulting from applying a move
    pub fn apply_move(&self, direction: Move) -> Coord {
        match direction {
            Move::Up => Coord { x: self.x, y: self.y + 1 },
            Move::Down => Coord { x: self.x, y: self.y - 1 },
            Move::Left => Coord { x: self.x - 1, y: self.y },
            Move::Right => Coord { x: self.x + 1, y: self.y },
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    pub id: String,
    pub ruleset: Ruleset,
    pub timeout: u32,
    // source: Option<String>, // Consider adding if needed based on API version
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ruleset {
    pub name: String,
    pub version: String,
    // settings: Option<serde_json::Value>, // Use generic Value for flexibility
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Battlesnake {
    pub id: String,
    pub name: String,
    pub health: u32,
    pub body: Vec<Coord>,
    pub head: Coord,
    pub length: u32,
    // latency: Option<String>,
    // shout: Option<String>,
    // squad: Option<String>,
    // customizations: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Board {
    pub height: i32,
    pub width: i32,
    pub food: Vec<Coord>,
    pub hazards: Vec<Coord>,
    pub snakes: Vec<Battlesnake>,
}

// Helper methods for Board
impl Board {
    // Checks if a coordinate is within the board boundaries
    pub fn in_bounds(&self, coord: &Coord) -> bool {
        coord.x >= 0 && coord.x < self.width && coord.y >= 0 && coord.y < self.height
    }

    // Checks if a coordinate is occupied by any snake body segment (excluding tails optionally)
    pub fn is_occupied(&self, coord: &Coord, exclude_tails: bool) -> bool {
        self.snakes.iter().any(|snake| {
            snake.body.iter().enumerate().any(|(i, segment)| {
                // If exclude_tails is true, skip the last segment (tail tip)
                if exclude_tails && i == snake.body.len() - 1 {
                    return false;
                }
                segment.x == coord.x && segment.y == coord.y
            })
        })
    }

    // Checks if a coordinate is occupied by any snake body segment, considering a specific snake ID
    pub fn is_occupied_by_snake(&self, coord: &Coord, snake_id: &str) -> bool {
        self.snakes.iter()
            .filter(|s| s.id == snake_id)
            .any(|snake| {
                snake.body.iter().any(|segment| {
                    segment.x == coord.x && segment.y == coord.y
                })
            })
    }

     // Checks if a coordinate is occupied by any snake other than the specified one
    pub fn is_occupied_by_others(&self, coord: &Coord, self_id: &str, exclude_tails: bool) -> bool {
        self.snakes.iter()
            .filter(|s| s.id != self_id)
            .any(|snake| {
                 snake.body.iter().enumerate().any(|(i, segment)| {
                    if exclude_tails && i == snake.body.len() - 1 {
                        return false;
                    }
                    segment.x == coord.x && segment.y == coord.y
                })
            })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    pub game: Game,
    pub turn: u32,
    pub board: Board,
    pub you: Battlesnake,
} 