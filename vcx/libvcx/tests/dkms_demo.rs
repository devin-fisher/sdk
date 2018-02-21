#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_json;
extern crate rand;
extern crate tempfile;
extern crate tempdir;
extern crate vcx;

use std::env;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time;
use std::path::Path;

mod utils;
use utils::wallet;
use utils::api_caller::str_r_panic;

mod dkms;
use dkms::constants::{DEV_GENESIS_NODE_TXNS, pairwise_info, wallet_entries, asset_name};
use dkms::actor::Actor;
use dkms::util::{print_chapter, find_actor};

use vcx::settings::DEFAULT_GENESIS_PATH;

fn chapter_1_demo(actor: &Actor) {
    print_chapter("CHAPTER ONE", None);

    match actor {
        &Actor::Alice => {
            println!("ENTER ALICE");
        },
        &Actor::Bob => (),
        &Actor::CUnion => (),
        &Actor::Dakota => (),
    }
}

fn chapter_2_demo(actor: &Actor) {
    print_chapter("CHAPTER TWO", None);

    match actor {
        &Actor::Alice => {
            println!("ENTER ALICE");
        },
        &Actor::Bob => (),
        &Actor::CUnion => (),
        &Actor::Dakota => (),
    }
}

fn chapter_3_demo(actor: &Actor) {
    print_chapter("CHAPTER THREE", None);

    match actor {
        &Actor::Alice => {
            println!("ENTER ALICE");
        },
        &Actor::Bob => (),
        &Actor::CUnion => (),
        &Actor::Dakota => (),
    }
}

fn chapter_4_demo(actor: &Actor) {
    print_chapter("CHAPTER FOUR", None);

    match actor {
        &Actor::Alice => {
            println!("ENTER ALICE");
        },
        &Actor::Bob => (),
        &Actor::CUnion => (),
        &Actor::Dakota => (),
    }
}

fn chapter_5_demo(actor: &Actor) {
    print_chapter("CHAPTER FIVE", None);

    match actor {
        &Actor::Alice => {
            println!("ENTER ALICE");
        },
        &Actor::Bob => (),
        &Actor::CUnion => (),
        &Actor::Dakota => (),
    }
}

fn init_pool() {
    let gen_file_path = Path::new(DEFAULT_GENESIS_PATH);
    println!("Writing genesis file at {:?}", gen_file_path);

    let mut gen_file = File::create(gen_file_path).unwrap();
    for line in DEV_GENESIS_NODE_TXNS {
        gen_file.write_all(line.as_bytes()).unwrap();
        gen_file.write_all("\n".as_bytes()).unwrap();
    }
    gen_file.flush().unwrap();
}

fn init_actor(actor: &Actor, dir: &Path) {
    print_chapter("INIT ACTOR", None);
    println!("Setuping {:?}'s wallet and configuration.", actor);
    wallet::add_wallet_entries(Some(asset_name(actor).as_str()),
                               wallet_entries(actor)
    ).unwrap();
    println!("Wallet Setup is done.");

    let _pairwise_info = pairwise_info(actor);
    let random_int: u32 = rand::random();
    let logo_url = format!("https://robohash.org/{}?set=set3", random_int);
    let wallet_name = asset_name(actor);
    let config = json!(
        {
           "pool_name":"dkms_pool",
           "config_name":"my_config",
           "wallet_name":wallet_name,
           "agency_pairwise_did":"72x8p4HubxzUK1dwxcc5FU",
           "agent_pairwise_did":"UJGjM6Cea2YVixjWwHN9wq",
           "enterprise_did_agency":"RF3JM851T4EQmhh8CdagSP",
           "enterprise_did_agent":"AB3JM851T4EQmhh8CdagSP",
           "enterprise_name":"enterprise",
           "agency_pairwise_verkey":"7118p4HubxzUK1dwxcc5FU",
           "agent_pairwise_verkey":"U22jM6Cea2YVixjWwHN9wq",
           "logo_url":logo_url,
           "wallet_key": ""
        }
    );

    println!("{}'s configuration", actor);
    let config_data = serde_json::to_string_pretty(&config).unwrap();
    println!("{}", config_data);

    let config_file_path = dir.join("config.json");
    let mut config_file = File::create(&config_file_path).unwrap();
    config_file.write_all(config_data.as_bytes()).unwrap();
    config_file.flush().unwrap();
    println!("Config Setup is done.");


    println!("Starting VCX!");
    str_r_panic(config_file_path.to_str().unwrap(), vcx::api::vcx::vcx_init);
    thread::sleep(time::Duration::from_millis(10));

}

fn full_demo(actor: &Actor) {
    init_pool();

    let dir = tempdir::TempDir::new(&asset_name(actor)).unwrap();
    {
        let dir_path = dir.path();
        println!("Using {:?} for {}", dir_path, actor);

        init_actor(actor, dir_path);
        chapter_1_demo(actor);
        chapter_2_demo(actor);
        chapter_3_demo(actor);
        chapter_4_demo(actor);
        chapter_5_demo(actor);

        print_chapter("DONE", None);
    }
    dir.close().unwrap();
}


#[test]
fn dkms_demo(){
    match env::var("DKMS_ACTOR"){
        Ok(_) => {
            let actor = find_actor();
            println!("\nRunning as '{:?}'", actor);
            full_demo(&actor)
        },
        Err(_) => {},
    }
}
