#[macro_use]
extern crate serde_derive;

mod errors;
mod music;
mod playlist;

use futures::future::Future;

use music::Client;
use playlist::read_playlist;

fn main() {
    env_logger::init();

    let client = Client::new(
        "TEAM_ID".to_owned(),
        "KEY_ID".to_owned(),
        std::fs::read_to_string("/path/to/private/key/file.p8").unwrap(),
        "us".to_owned(),
    );

    tokio::run(
        client
            .search(&["nallai allai"])
            .map(|result| {
                let data = result.data();
                println!("{:#?}", data);
            })
            .map_err(|err| eprintln!("ERROR: {:#?}", err)),
    );

    //    let entries = read_playlist("./sample/playlist.json").unwrap();
    //    println!("{:#?}", entries);
}
