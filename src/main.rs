#![feature(proc_macro_hygiene, decl_macro)]

const VERSION_MAJOR: u32 = 0;
const VERSION_MINOR: u32 = 1;

extern crate nalgebra;
extern crate simplelog;
#[macro_use] extern crate log;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde_derive;

mod ratings;
mod dataprovider;
mod recommender;

use std::sync::Arc;
use rocket::{State, http::Status};
use rocket_contrib::json::JsonValue;
use simplelog::{TermLogger, LevelFilter, Level};
use self::dataprovider::TestDataCsvProvider;
use self::recommender::{RecommendationEngine, PredictionError};

fn print_hello() {
    let release_debug_flag = if cfg!(debug_assertions) { "d" } else { "r" };

    println!("8888888b.                                             888                        
888   Y88b                                            888                        
888    888                                            888                        
888   d88P  .d88b.   .d8888b  .d88b.          .d8888b 88888b.   8888b.  88888b.  
8888888P\"  d8P  Y8b d88P\"    d88\"\"88b        d88P\"    888 \"88b     \"88b 888 \"88b 
888 T88b   88888888 888      888  888 888888 888      888  888 .d888888 888  888 
888  T88b  Y8b.     Y88b.    Y88..88P        Y88b.    888  888 888  888 888  888 
888   T88b  \"Y8888   \"Y8888P  \"Y88P\"          \"Y8888P 888  888 \"Y888888 888  888 v{}.{}{}",
    VERSION_MAJOR, VERSION_MINOR, release_debug_flag);
    println!("(tsundere edition)");
    println!();
}

fn main() {
    let dataprovider = TestDataCsvProvider::new("/tmp/recommendations");
    let recom_engine = RecommendationEngine::new_default(dataprovider);
    
    print_hello();
    // Initialize logging
    let mut log_config = simplelog::Config::default();
    log_config.target = Some(Level::Info);
    TermLogger::init(LevelFilter::Trace, log_config).unwrap();
    info!(target: "Reco-Chan", "Executing initial training round...");
    info!(target: "Reco-Chan", "I'm not doing this for you though, I'm doing this because I want to! (,,Ծ‸Ծ,, )");

    // Train initial round before starting web-server
    recom_engine.retrain();

    info!(target: "Reco-Chan", "Initial training has finished. If you ask me for recommendations now, I MAY tell you the answer. But only reluctantly! ヽ(*≧ω≦)ﾉ");

    rocket::ignite()
            .manage(Arc::new(recom_engine))
            .mount("/", routes![personal_recommendation])
            .launch();
}


#[get("/personal/<userid>")]
fn personal_recommendation(userid: u64, recom_engine: State<Arc<RecommendationEngine>>) -> Result<JsonValue, Status> {
    match recom_engine.predict_for_user(userid) {
        Ok(prediction) => {
            return Ok(json!(prediction));
        },
        Err(e) => {
            match e {
                PredictionError::NotInitialized | PredictionError::Unknown => {
                    return Err(Status::new(500, "Some weird mistake occured, sorry!"));
                },
                PredictionError::UnknownUser => {
                    return Err(Status::new(404, "I can not yet predict something for this user, sorry!"));
                }
            }
        }
    }
}
