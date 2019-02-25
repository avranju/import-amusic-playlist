#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

mod errors;
mod music;
mod playlist;

use futures::future::Future;
use structopt::StructOpt;

use music::Client;
use playlist::read_playlist;

#[derive(StructOpt, Debug)]
#[structopt(name = "import-amusic-playlist")]
struct Opt {
    #[structopt(short = "t", long = "team-id")]
    pub team_id: String,

    #[structopt(short = "k", long = "key-id")]
    pub key_id: String,

    #[structopt(short = "p", long = "private-key-path")]
    pub private_key_path: String,

    #[structopt(short = "s", long = "storefront", default_value = "us")]
    pub storefront: String,
}

fn main() {
    env_logger::init();

    let opt = Opt::from_args();

    let client = Client::new(
        opt.team_id,
        opt.key_id,
        std::fs::read_to_string(opt.private_key_path).unwrap(),
        opt.storefront,
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
