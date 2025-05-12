use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use log::{info, error};
use rand::seq::SliceRandom;
use serde::Serialize;

mod game_state;
mod logic;
mod sim;

use game_state::{GameState, Move};

// ---------------------------
// Data structures
// ---------------------------
#[derive(Serialize, Debug)]
pub struct Game {
    pub id: String,
    pub ruleset: Ruleset,
    pub timeout: u32,
}

#[derive(Serialize, Debug)]
pub struct Ruleset {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Debug, Clone)]
pub struct Battlesnake {
    pub id: String,
    pub name: String,
    pub health: u32,
    pub body: Vec<Coord>,
    pub head: Coord,
    pub length: u32,
}

#[derive(Serialize, Debug)]
pub struct Board {
    pub height: i32,
    pub width: i32,
    pub food: Vec<Coord>,
    pub hazards: Vec<Coord>,
    pub snakes: Vec<Battlesnake>,
}

#[derive(Serialize, Debug)]
pub struct GameState {
    pub game: Game,
    pub turn: u32,
    pub board: Board,
    pub you: Battlesnake,
}

// ---------------------------
// API responses
// ---------------------------
#[derive(Serialize)]
struct InfoResponse {
    apiversion: String,
    author: String,
    color: String,
    head: String,
    tail: String,
    version: String,
}

#[derive(Serialize)]
struct MoveResponse {
    #[serde(rename = "move")]
    move_dir: String,
    shout: String,
}

// ---------------------------
// Handlers
// ---------------------------
#[get("/")]
async fn handle_index() -> impl Responder {
    HttpResponse::Ok().json(InfoResponse {
        apiversion: "1".to_string(),
        author: "YourName".to_string(),
        color: "#FF5733".to_string(),
        head: "default".to_string(),
        tail: "default".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[post("/start")]
async fn handle_start(state: web::Json<GameState>) -> impl Responder {
    info!("Game {} started. Ruleset: {}", state.game.id, state.game.ruleset.name);
    HttpResponse::Ok().body("")
}

#[post("/move")]
async fn handle_move(state: web::Json<GameState>) -> impl Responder {
    let game_id = &state.game.id;
    let turn = state.turn;
    info!("Game {} Turn {}", game_id, turn);

    let chosen_move = match logic::decide_move(&state) {
        Ok(m) => {
            info!("Game {} Turn {}: Chose move {}", game_id, turn, m.as_str());
            m
        },
        Err(e) => {
            error!("Game {} Turn {}: Error deciding move: {}. Falling back to 'down'.", game_id, turn, e);
            Move::Down
        }
    };

    HttpResponse::Ok().json(MoveResponse {
        move_dir: chosen_move.as_str().to_string(),
        shout: format!("Turn {}!", turn),
    })
}

#[post("/end")]
async fn handle_end(state: web::Json<GameState>) -> impl Responder {
    let outcome = if state.board.snakes.iter().any(|s| s.id == state.you.id) {
        if state.board.snakes.len() == 1 {
            "Win"
        } else {
            "Survived?"
        }
    } else {
        "Loss/Draw"
    };
    info!("Game {} ended. Outcome: {}", state.game.id, outcome);
    HttpResponse::Ok().body("")
}

// ---------------------------
// Server setup
// ---------------------------
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    info!("{} v{} starting on {}", 
        env!("CARGO_PKG_NAME"), 
        env!("CARGO_PKG_VERSION"), 
        addr);

    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .service(handle_index)
            .service(handle_start)
            .service(handle_move)
            .service(handle_end)
    })
    .bind(addr)?
    .run()
    .await
} 