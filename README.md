Okay, let's break down building a Battlesnake API in Rust and discuss strategies for competitive play, potentially aiming for a tournament like the one you mentioned (though "PHPTek" isn't a standard Battlesnake tournament name, the principles apply to any competitive setting).

We'll use the official Rust starter snake as a base, which utilizes the `actix-web` framework and `serde` for JSON handling.

**1. Rust Battlesnake API Implementation (Based on Starter)**

The core idea is to create a web server that responds to specific POST requests from the Battlesnake engine at defined endpoints.

**Dependencies (`Cargo.toml`)**

Make sure your `Cargo.toml` includes the necessary dependencies:

```toml
[package]
name = "my-battlesnake"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4" # Or the latest version
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8" # For random moves initially, can be removed later
log = "0.4"
env_logger = "0.9"
```

**Core Logic (`src/main.rs`)**

Here's a simplified structure based on the official starter, incorporating the necessary endpoints and basic data structures:

```rust
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom; // For basic random moves

// --- Data Structures (matching Battlesnake API) ---
#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    id: String,
    ruleset: Ruleset,
    timeout: u32,
    source: String, // Added in later API versions, indicates origin (tournament, league, etc.)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ruleset {
    name: String,
    version: String,
    settings: RulesetSettings,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RulesetSettings {
    food_spawn_chance: u32,
    minimum_food: u32,
    hazard_damage_per_turn: u32,
    // Add other settings based on the specific ruleset if needed
    // e.g., royale: Option<RoyaleSettings>, squad: Option<SquadSettings>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Coord {
    x: i32,
    y: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Battlesnake {
    id: String,
    name: String,
    health: u32,
    body: Vec<Coord>,
    latency: String, // Often a string representing milliseconds e.g. "123"
    head: Coord,
    length: u32,
    shout: String,
    squad: String, // Squad ID if applicable
    customizations: Option<SnakeCustomizations>, // Optional customizations
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnakeCustomizations {
    color: String,
    head: String,
    tail: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Board {
    height: i32,
    width: i32,
    food: Vec<Coord>,
    hazards: Vec<Coord>,
    snakes: Vec<Battlesnake>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    game: Game,
    turn: u32,
    board: Board,
    you: Battlesnake,
}

// --- API Responses ---

#[derive(Serialize, Debug)]
pub struct InfoResponse {
    apiversion: String,
    author: String,
    color: String,
    head: String,
    tail: String,
    version: String,
}

#[derive(Serialize, Debug)]
pub struct MoveResponse {
    #[serde(rename = "move")]
    move_dir: String, // "up", "down", "left", "right"
    shout: String,
}

// --- Request Handlers ---

#[get("/")]
async fn handle_index() -> impl Responder {
    info!("GET / - Info");
    HttpResponse::Ok().json(InfoResponse {
        apiversion: "1".to_string(),
        author: "YourName".to_string(),
        color: "#FF0000".to_string(), // Choose your color
        head: "default".to_string(),  // Choose head type
        tail: "default".to_string(),  // Choose tail type
        version: "0.1.0".to_string(),
    })
}

#[post("/start")]
async fn handle_start(state: web::Json<GameState>) -> impl Responder {
    info!("POST /start - Game ID: {}", state.game.id);
    // You can initialize game-specific state here if needed
    HttpResponse::Ok().body("") // Must return HTTP 200 OK
}

#[post("/move")]
async fn handle_move(state: web::Json<GameState>) -> impl Responder {
    info!("POST /move - Turn: {}", state.turn);

    // --- Basic Safety Logic (Example) ---
    let possible_moves = vec!["up", "down", "left", "right"];
    let mut safe_moves = Vec::new();

    let my_head = &state.you.head;
    let my_body = &state.you.body;
    let board_width = state.board.width;
    let board_height = state.board.height;

    for &m_str in &possible_moves {
        let target_coord = match m_str {
            "up" => Coord { x: my_head.x, y: my_head.y + 1 },
            "down" => Coord { x: my_head.x, y: my_head.y - 1 },
            "left" => Coord { x: my_head.x - 1, y: my_head.y },
            "right" => Coord { x: my_head.x + 1, y: my_head.y },
            _ => my_head.clone(), // Should not happen
        };

        let mut is_safe = true;

        // 1. Check Wall Collisions
        if target_coord.x < 0 || target_coord.x >= board_width || target_coord.y < 0 || target_coord.y >= board_height {
            is_safe = false;
        }

        // 2. Check Self Collision (excluding tail in some scenarios, but be careful!)
        // Note: The Battlesnake engine *might* move the tail before your next move,
        //       so colliding with your own tail *can* be safe, but it's complex.
        //       Simplest safe approach: avoid all body parts.
        if is_safe {
            for (i, segment) in my_body.iter().enumerate() {
                 // Don't check the very last segment (tail tip) if length > 1, as it will move away
                if i < my_body.len() -1 && target_coord.x == segment.x && target_coord.y == segment.y {
                    is_safe = false;
                    break;
                }
            }
        }

        // 3. Check Other Snake Collisions
        if is_safe {
            for snake in &state.board.snakes {
                // Don't check collision with self again
                if snake.id == state.you.id { continue; }

                // Check collision with body parts (including their head)
                for (i, segment) in snake.body.iter().enumerate() {
                     // Allow potential head-to-head if we are longer (Risky! Needs better logic)
                    // For basic safety, avoid all parts of other snakes.
                    // Exception: Avoid the tail tip unless it's their head (snake length 1)
                    let is_tail_tip = i == snake.body.len() - 1;
                     if target_coord.x == segment.x && target_coord.y == segment.y && !is_tail_tip {
                         is_safe = false;
                         break; // Stop checking this snake
                     }
                     // More complex: Check head-to-head scenarios only if safe
                     if target_coord.x == segment.x && target_coord.y == segment.y && is_tail_tip && segment.x == snake.head.x && segment.y == snake.head.y {
                        // This is their head (could be a head-on collision)
                        // Basic avoidance: Consider it unsafe for now.
                         is_safe = false;
                         break;
                     }
                }
                if !is_safe { break; } // Stop checking other snakes if already unsafe
            }
        }

        // 4. Check Hazard Collision (optional, depending on ruleset)
        // if is_safe {
        //     for hazard in &state.board.hazards {
        //         if target_coord.x == hazard.x && target_coord.y == hazard.y {
        //             // Decide if moving into hazard is acceptable (e.g., low damage vs. guaranteed death)
        //             // For now, let's avoid them
        //             is_safe = false;
        //             break;
        //         }
        //     }
        // }


        if is_safe {
            safe_moves.push(m_str.to_string());
        }
    }

    // --- Choose Move ---
    let chosen_move = if safe_moves.is_empty() {
        info!("No safe moves detected! Moving down as default.");
        "down".to_string() // Default or panic - ideally should not happen with good logic
    } else {
        // Basic Strategy: Choose a random safe move
        // TODO: Replace this with actual strategy!
        let mut rng = rand::thread_rng();
        safe_moves.choose(&mut rng).unwrap_or(&"down".to_string()).clone()
    };

    info!("MOVE: {}", chosen_move);
    HttpResponse::Ok().json(MoveResponse {
        move_dir: chosen_move,
        shout: "Moving!".to_string(),
    })
}

#[post("/end")]
async fn handle_end(state: web::Json<GameState>) -> impl Responder {
    info!("POST /end - Game ID: {} finished. Win: {}", state.game.id, state.board.snakes.iter().any(|s| s.id == state.you.id));
    // Clean up game state or log results
    HttpResponse::Ok().body("")
}

// --- Main Server Setup ---
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("0.0.0.0:{}", port);

    info!("Starting Battlesnake server on {}...", address);

    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default()) // Log requests
            .service(handle_index)
            .service(handle_start)
            .service(handle_move)
            .service(handle_end)
    })
    .bind(address)?
    .run()
    .await
}
```

**How to Run:**

1.  Save the code as `src/main.rs`.
2.  Save the dependencies in `Cargo.toml`.
3.  Run `cargo build` to compile.
4.  Run `cargo run` to start the server.
5.  You can test it locally using the Battlesnake CLI or by creating a snake on play.battlesnake.com pointing to your server's address (you might need a tool like `ngrok` to expose your local server to the internet).

**2. Strategy for Winning ("Best Snake")**

There is no single "best" snake, as the optimal strategy depends heavily on the ruleset, the behaviour of opponents, and the current "meta" on leaderboards or in tournaments. However, here are key areas to focus on, moving from basic survival to advanced tactics:

*   **Survival is Paramount:**
    *   **Avoid Walls:** Never move off the board. Your basic code should handle this.
    *   **Avoid Self-Collision:** Don't run into your own body. The basic code handles this simplistically; advanced snakes might consider moving onto their tail square *if* it's guaranteed to move away next turn.
    *   **Avoid Other Snakes (Initially):** Don't collide with other snakes' bodies. Be very careful about head-to-head collisions – only engage if you are *certain* you are longer *and* the move is safe.

*   **Basic Needs & Tactics:**
    *   **Seek Food (When Necessary):** Go for food if your health is getting low (e.g., below 50) or if it's nearby and safe. Don't starve!
    *   **Avoid Obvious Traps:** Don't move into a space where you have no escape routes on the *next* turn (e.g., a small dead-end). Look one step ahead.
    *   **Center Control (Often Good):** Staying near the center generally gives more movement options than hugging the edges.

*   **Intermediate Strategies:**
    *   **Space Control:** Try to move in ways that cut off parts of the board, limiting the options for other snakes.
    *   **Flood Fill / Voronoi:** Use algorithms like flood fill to determine how much space is accessible from a given square. This helps identify safe areas and potential traps for opponents.
    *   **Targeting Food Competitively:** If you and another snake are going for the same food, assess if you can reach it first and safely.
    *   **Basic Opponent Modeling:** Assume opponents will try to survive. Predict their likely safe moves to avoid collisions or trap them.

*   **Advanced Strategies (Needed for Top Competition):**
    *   **Search Algorithms:**
        *   **Minimax / Alpha-Beta Pruning:** Explore possible future game states, assuming opponents play optimally against you. Choose the move that leads to the best outcome in the worst-case scenario. This requires a good evaluation function (how "good" is a board state?).
        *   **Monte Carlo Tree Search (MCTS):** Use random simulations (playouts) to estimate the value of different moves. Often performs well in games with high branching factors like Battlesnake.
    *   **Heuristics:** Develop a robust evaluation function for your search algorithm. Consider factors like: health, length advantage, controlled space, distance to food, proximity to hazards, potential threats from opponents, number of available moves (mobility).
    *   **Opponent Prediction:** Model opponent behaviour more accurately. Do they prioritize food? Are they aggressive? Do they have known patterns?
    *   **Ruleset Adaptation:** Tailor your strategy to specific rulesets (e.g., Wrapped maps, Royale shrinking hazards, Constrictor mode). For example, in Royale, controlling space near the center becomes crucial as the hazards close in.
    *   **Latency Considerations:** In very competitive settings, server latency can matter. Your snake needs to respond well within the timeout (often 500ms). Optimize your code!

**Winning a Tournament:**

1.  **Understand the Format:** Is it a ladder, Swiss, or bracket? What are the specific rulesets being used?
2.  **Build a Robust Snake:** Focus on survival first, then add layers of heuristics and potentially search.
3.  **Test Extensively:** Use the Battlesnake CLI, run local games against other snakes (including previous versions of your own), and participate in the global ladder.
4.  **Analyze Losses:** Watch replays. Why did your snake lose? Was it a bad heuristic? A missed threat? A bug?
5.  **Iterate:** Continuously refine your logic based on testing and analysis.