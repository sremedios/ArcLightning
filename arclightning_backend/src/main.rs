extern crate futures;
extern crate hyper;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate toml;

use futures::future;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// after this concept is further understood, will switch to 'Either'
type ResponseFuture = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

// using PartialEq for unit tests
// Using clone in a unit test atm.  Might not be necessary
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Game {
    name: String,
    description: String,
    genres: Vec<String>,
    thumbnail_path: PathBuf,
    exe_path: PathBuf,
}

fn router(request: Request<Body>) -> ResponseFuture {
    let mut response = Response::new(Body::empty());

    match (request.method(), request.uri().path()) {
        (&Method::GET, "/api/v1/list_games") => {
            *response.body_mut() = Body::from("test");
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }
    Box::new(future::ok(response))
}

fn toml_to_hashmap(toml_filepath: PathBuf) -> Result<HashMap<String, Game>, Error> {
    let mut games_toml = String::new();
    let mut file = File::open(&toml_filepath)?;
    file.read_to_string(&mut games_toml)?;

    // error casting for homogeneous errors
    toml::from_str(&games_toml).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
}

fn main() {
    // Read initial games toml config
    let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect();

    // Store games locally on server
    let games: HashMap<String, Game> = toml_to_hashmap(toml_filepath).unwrap();
    let games_data = Arc::new(Mutex::new(games.clone()));

    // Host server
    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(router))
        .map_err(|err| eprintln!("server error: {}", err));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_json_serialization() {
        // Read in a specific file
        let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect();
        let games: HashMap<String, Game> = toml_to_hashmap(toml_filepath);

        // serialize as json
        let json_object_touhou = serde_json::to_string(&games.get("touhou_123")).unwrap();
        let json_object_melty_blood = serde_json::to_string(&games.get("melty_blood")).unwrap();

        // test cases separately to get around the nondeterministic order for hashmap
        let mut test_json_touhou = "{\"name\":\"Touhou\",\
                                    \"description\":\"bullet hell with waifus\",\
                                    \"genres\":[\"bullet hell\",\"anime\"],\
                                    \"thumbnail_path\":\"path/to/touhou/thumbnail\",\
                                    \"exe_path\":\"C:\\\\Users\\\\THISUSER\\\\TOUHOU_PATH\"}";
        let mut test_json_mb = "{\"name\":\"Melty Blood\",\
                                \"description\":\"fighter with waifus\",\
                                \"genres\":[\"fighter\",\"anime\",\"2d\"],\
                                \"thumbnail_path\":\"path/to/melty_blood/thumbnail\",\
                                \"exe_path\":\"C:\\\\Users\\\\THISUSER\\\\MELTY_BLOOD_PATH\"}";

        assert_eq!(json_object_touhou, test_json_touhou);
        assert_eq!(json_object_melty_blood, test_json_mb);
    }

    #[test]
    fn test_games_serialization() {
        // Read in a specific file
        let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect();
        let games: HashMap<String, Game> = toml_to_hashmap(toml_filepath);

        let games_clone = games.clone();

        // wrap all the games in a mutex
        // note that this moves games into the mutex
        let games_data = Arc::new(Mutex::new(games));

        assert_eq!(games_clone, *games_data.lock().unwrap());
    }

    #[test]
    fn test_read_toml() {
        // Read in a specific file
        let toml_filepath: PathBuf = ["test_files", "test_games.toml"].iter().collect();
        let games: HashMap<String, Game> = toml_to_hashmap(toml_filepath);

        let mut test_games: HashMap<String, Game> = HashMap::new();
        test_games.insert(
            "touhou_123".to_owned(),
            Game {
                name: "Touhou".to_owned(),
                description: "bullet hell with waifus".to_owned(),
                genres: vec!["bullet hell".to_owned(), "anime".to_owned()],
                thumbnail_path: PathBuf::from(r"path\to\touhou\thumbnail"),
                exe_path: PathBuf::from(r"C:\Users\THISUSER\TOUHOU_PATH"),
            },
        );

        test_games.insert(
            "melty_blood".to_owned(),
            Game {
                name: "Melty Blood".to_owned(),
                description: "fighter with waifus".to_owned(),
                genres: vec!["fighter".to_owned(), "anime".to_owned(), "2d".to_owned()],
                thumbnail_path: PathBuf::from(r"path\to\melty_blood\thumbnail"),
                exe_path: PathBuf::from(r"C:\Users\THISUSER\MELTY_BLOOD_PATH"),
            },
        );
        assert_eq!(games, test_games);
    }
}
