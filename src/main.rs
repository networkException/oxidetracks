mod storage;
mod location;

use anyhow::Result;
use chrono::{DateTime, Utc};
use log::{info, error};
use macros::IntoJsonResponse;

use location::Location;
use storage::Storage;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use std::{sync::{Arc, Mutex}, path::PathBuf, time::Instant};

use git_version::git_version;
use axum::{
    routing::get,
    Router,
    Server,
    extract::{State, Query}, Json
};

use serde::{Serialize, Deserialize};
use clap::Parser;

#[derive(Serialize, IntoJsonResponse)]
struct VersionResponse {
    version: String,
    git: String,
}

async fn get_version() -> VersionResponse {
    VersionResponse {
        version: "0.1.0".to_owned(),
        git: git_version!().to_owned()
    }
}

#[derive(Deserialize)]
struct ListQuery {
    user: Option<String>,
}

#[derive(Serialize, IntoJsonResponse)]
struct ListResponse {
    results: Vec<String>,
}

#[derive(Serialize, IntoJsonResponse)]
struct ErrorResponse {
    error: String,
}

impl ErrorResponse {
    fn new(error: &str) -> ErrorResponse {
        ErrorResponse { error: error.to_owned() }
    }
}

async fn get_list(State(app_state): State<AppState>, Query(query): Query<ListQuery>) -> Result<ListResponse, ErrorResponse> {
    let storage = &app_state.storage.lock()
        .map_err(|_| ErrorResponse::new("Unable to take lock for in memory storage"))?;

    match &query.user {
        Some(user) => {
            match storage.user(user) {
                Some(user_storage) => Ok(ListResponse { results: user_storage.device_names() }),
                // Mirroring what owntracks/recorder would do
                None => Err(ErrorResponse { error: "Cannot open requested directory".to_string() })
            }
        }
        None => Ok(ListResponse { results: storage.user_names() })
    }
}

type LastResponse = Json<Vec<Location>>;

async fn get_last(State(app_state): State<AppState>) -> Result<LastResponse, ErrorResponse> {
    let storage = &app_state.storage.lock()
        .map_err(|_| ErrorResponse::new("Unable to take lock for in memory storage"))?;

    Ok(Json(storage.users().iter()
        .flat_map(|(user_name, user_storage)| user_storage.devices().iter()
            .filter(|(_, device_storage)| device_storage.last_location().is_some())
            .map(|(device_name, device_storage)| device_storage.last_location().clone())
            .flatten()
            .map(|location| location.clone()))
        .collect()))
}

#[derive(Deserialize)]
struct LocationsQuery {
    #[serde(with = "iso_date_format")]
    from: DateTime<Utc>,

    #[serde(with = "iso_date_format")]
    to: DateTime<Utc>,

    #[serde(rename = "user")]
    user_name: String,

    #[serde(rename = "device")]
    device_name: String,
    format: String
}

#[derive(Serialize, IntoJsonResponse)]
struct LocationsResponse {
    count: usize,
    data: Vec<Location>,
    // Always 200
    status: u16,
}

async fn get_locations(State(app_state): State<AppState>, Query(query): Query<LocationsQuery>) -> Result<LocationsResponse, ErrorResponse> {
    let storage = &app_state.storage.lock()
        .map_err(|_| ErrorResponse::new("Unable to take lock for in memory storage"))?;

    let started_fetching = Instant::now();

    let locations: Vec<Location> = storage.user(query.user_name.as_str())
        .and_then(|user_storage| user_storage.device(query.device_name.as_str()))
        .map(|device_storage| device_storage.locations()
            .iter()
            .skip_while(|location| location.timestamp <= query.from)
            .take_while(|location| location.timestamp <= query.to)
            .map(Location::clone)
            .collect())
        .unwrap_or(vec![]);

    info!(target: "API", "Fetched {} locations in {:.2?}", locations.len(), started_fetching.elapsed());

    Ok(LocationsResponse {
        count: locations.len(),
        status: 200,
        data: locations,
    })
}

#[derive(Clone)]
struct AppState {
    storage: Arc<Mutex<Storage>>
}

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// The path to storage
    #[arg(short, long, env)]
    storage_path: PathBuf,

    /// The address to bind to
    #[clap(short, long, env, default_value = "[::]:3000")]
    bind: String,
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"));

    let Arguments { storage_path, bind } = Arguments::parse();

    let mut storage = Storage::new(storage_path);

    storage.read_from_fs().unwrap();

    let state = AppState {
        storage: Arc::new(Mutex::new(storage))
    };

    let app = Router::new()
        .route("/api/0/version", get(get_version))
        .route("/api/0/list", get(get_list))
        .route("/api/0/last", get(get_last))
        .route("/api/0/locations", get(get_locations))
        .with_state(state)
        .layer(ServiceBuilder::new()
            .layer(CorsLayer::permissive()));

    let Ok(socket_address) = bind.parse() else {
        error!(target: "API", "Unable to parse '{}' as socket address", bind);
        return;
    };

    info!(target: "API", "Listening on {}", socket_address);

    Server::bind(&socket_address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

mod iso_date_format {
    use chrono::{DateTime, Utc, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S";

    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}
