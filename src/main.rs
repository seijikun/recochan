#![feature(proc_macro_hygiene, decl_macro)]

const VERSION_MAJOR: u32 = 0;
const VERSION_MINOR: u32 = 1;

extern crate kdtree;
extern crate config;
extern crate nalgebra;
extern crate simplelog;
extern crate serde_derive;
#[macro_use] extern crate log;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;

mod ratings;
mod settings;
mod dataprovider;
mod recommender;

use std::thread;
use std::sync::Arc;
use rocket::{State, http::Status};
use rocket_contrib::json::JsonValue;
use simplelog::{TermLogger, TerminalMode, LevelFilter, Level};
use self::dataprovider::*;
use self::settings::RecoChanSettingsDataProvider;
use self::recommender::{RecommendationEngine, PredictionError};

// Change log-level depending on build-type for now
#[cfg(not(debug_assertions))]
const LOGLEVEL: LevelFilter = LevelFilter::Info;
#[cfg(debug_assertions)]
const LOGLEVEL: LevelFilter = LevelFilter::Trace;

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
    // Load and parse configuration file
    let settings = match settings::RecoChanSettings::open("recochan.json") {
        Ok(settings) => settings,
        Err(e) => {
            eprintln!("{}", e);
            panic!();
        }
    };

    // Instantiate configured dataprovider
    let dataprovider: Box<dyn RatingDataProvider + Send + Sync> = match settings.dataprovider {
        RecoChanSettingsDataProvider::SQL { connection_string, where_clause, aid_name, uid_name, rating_name, table_name } => {
            Box::new(SQLDataProvider::new(&connection_string, &where_clause, &aid_name, &uid_name, &rating_name, &table_name))
        }
        RecoChanSettingsDataProvider::TestCSV { path } => Box::new(TestDataCsvProvider::new(&path))
    };

    info!(target: "Reco-Chan", "I'm applying the configuration you gave me, but only because I got nothing else to do!");

    // Create recommendation engine using configured dataprovider
    let recom_engine = Arc::new(RecommendationEngine::new_default(dataprovider));

    print_hello();
    // Initialize logging
    let mut log_config = simplelog::Config::default();
    log_config.target = Some(Level::Info);
    TermLogger::init(LOGLEVEL, log_config, TerminalMode::Mixed).unwrap();
    info!(target: "Reco-Chan", "Executing initial training round...");
    info!(target: "Reco-Chan", "I'm not doing this for you though, I'm doing this because I want to! (,,Ծ‸Ծ,, )");

    // Train initial round before starting web-server
    recom_engine.retrain();

    info!(target: "Reco-Chan", "Initial training has finished. If you ask me for recommendations now, I MAY tell you the answer. But only reluctantly! ヽ(*≧ω≦)ﾉ");

    // Start thread that will do the periodical re-training
    let (recom_engine_clone, retrain_every_sec) = (recom_engine.clone(), settings.retrain_every_sec);
    info!(target: "Reco-Chan", "Maybe I will remember to retrain every {}sec. But I will probably forget.", retrain_every_sec);
    thread::spawn(move || {
        let recom_engine = &recom_engine_clone;
        thread::sleep(std::time::Duration::from_secs(retrain_every_sec));
        info!(target: "Reco-Chan", "Ugh, I can't believe I actually remembered that you asked me to retrain now.");
        recom_engine.retrain();
    });

    // Configure and startup Web-API
    let api_env = if cfg!(debug_assertions) { rocket::config::Environment::Development } else { rocket::config::Environment::Production };
    let api_config = rocket::config::Config::build(api_env)
                        .address(settings.api.bind)
                        .port(settings.api.port)
                        .finalize()
                        .expect("Failed to configure Web-Service");

    rocket::custom(api_config)
            .manage(recom_engine)
            .mount("/", routes![
                endpoint_personal_recommendation,
                endpoint_similar_users,
                endpoint_similar_animes
            ])
            .launch();
}


#[get("/users/<userid>/recommend")]
fn endpoint_personal_recommendation(userid: u64, recom_engine: State<Arc<RecommendationEngine>>) -> Result<JsonValue, Status> {
    match recom_engine.predict_user_ratings(userid) {
        Ok(prediction) => {
            return Ok(json!(prediction));
        },
        Err(e) => {
            match e {
                PredictionError::UnknownUser => {
                    return Err(Status::new(404, "I can not yet predict something for this user, sorry!"));
                },
                _ => {
                    return Err(Status::new(500, "Some weird mistake occured, sorry!"));
                }
            }
        }
    }
}

#[get("/users/<userid>/similar?<count>")]
fn endpoint_similar_users(userid: u64, count: Option<usize>, recom_engine: State<Arc<RecommendationEngine>>) -> Result<JsonValue, Status> {
    match recom_engine.find_k_similar_users(userid, count.unwrap_or(5)) {
        Ok(similar_users) => {
            return Ok(json!(similar_users));
        },
        Err(e) => {
            match e {
                PredictionError::UnknownUser => {
                    warn!(target: "Reco-Chan", "User unknown: {}", userid);
                    return Err(Status::NotFound);
                },
                _ => {
                    return Err(Status::raw(500));
                }
            }
        }
    }
}

#[get("/animes/<animeid>/similar?<count>")]
fn endpoint_similar_animes(animeid: u64, count: Option<usize>, recom_engine: State<Arc<RecommendationEngine>>) -> Result<JsonValue, Status> {
    match recom_engine.find_k_similar_animes(animeid, count.unwrap_or(5)) {
        Ok(similar_animes) => {
            return Ok(json!(similar_animes));
        },
        Err(e) => {
            match e {
                PredictionError::UnknownAnime => {
                    warn!(target: "Reco-Chan", "Anime unknown: {}", animeid);
                    return Err(Status::NotFound);
                },
                _ => {
                    return Err(Status::raw(500));
                }
            }
        }
    }
}
