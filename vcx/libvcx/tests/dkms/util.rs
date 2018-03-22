extern crate serde_json;

use super::actor::Actor;
use std::env;
use std::thread;
use std::time;
use serde_json::Value;

use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::path::Path;
use std::fs::remove_file;
use std::io;

const LONG_STAR: usize = 64;
const SHORT_STAR: usize = 5;

pub fn print_chapter(chap_name: &str, line_len: Option<usize>) {
    let line_len = line_len.unwrap_or(LONG_STAR);
    let short_star_len = SHORT_STAR;
    let short_space_len = (line_len - (2*short_star_len) - chap_name.len())/2;

    let line = format!("{star}{space}{name}{space}{star}"
                       , star="*".repeat(short_star_len)
                       , space=" ".repeat(short_space_len)
                       , name=chap_name);

    println!("\n{}\n","*".repeat(line_len));
    print!("{}", line);
    if line.len() < line_len {
        print!("*\n");
    }
        else {
            print!("\n");
        }

    println!("\n{}\n","*".repeat(line_len));

}

pub fn find_actor() -> Actor{
    match env::var("DKMS_ACTOR"){
        Ok(actor) => {
            match actor.to_lowercase().as_ref() {
                "alice" => Actor::Alice,
                "bob" => Actor::Bob,
                "cunion" => Actor::CUnion,
                "dakota" => Actor::Dakota,
                "alice_new" => Actor::AliceNew,
                _ => panic!("Actor '{}' is not a valid option", actor)
            }
        },
        Err(_) => {
            panic!("Actor was not specified!")
        },
    }
}

pub fn pr_json(val: &str) {
    let j: Value = serde_json::from_str(val).unwrap();

    println!("{}", serde_json::to_string_pretty(&j).unwrap())
}

#[allow(dead_code)]
pub fn long_sleep() {
    let ten_s = time::Duration::from_secs(15);
    thread::sleep(ten_s);
}

pub fn should_print_wait_msg(msg: &str, val: usize, threshold: usize) -> usize {
    if val == threshold {
        println!("{}", msg);
        return 0;
    }
    else {
        return val + 1;
    }
}

pub fn send_via_file(data: &str, path: &Path, _timeout: Option<u32>) -> Result<(), ()> {
    let mut f = File::create(path).unwrap();
    let mut should_print = 0;
    f.write_all(data.as_bytes()).or(Err(()))?;

    loop {
        if !path.exists() {
            break;
        }

        should_print =should_print_wait_msg("Waiting for invite to be taken!",
                                            should_print,
                                            8);
        thread::sleep(time::Duration::from_secs(1));
    }

    Ok(())
}


pub fn receive_via_file(path: &Path, _timeout: Option<u32>) -> Result<String, ()> {
    let mut should_print = 0;
    loop {
        if path.exists() {
            let mut f = File::open(path).unwrap();
            let mut rtn = String::new();
            f.read_to_string(&mut rtn).unwrap();

            remove_file(path).unwrap();
            return Ok(rtn);
        }

        should_print =should_print_wait_msg("Waiting for invite!",
                                            should_print,
                                            8);

        thread::sleep(time::Duration::from_secs(1));
    }
}

pub fn gate(msg: Option<&str>, use_gate: bool) {
    if !use_gate {
        return;
    }

    println!("\n{}\n","*".repeat(LONG_STAR));
    match msg {
        None => {
            print!("Press any enter to continue . . .")
        }
        Some(m) => {
            print!("{}", m);
        }
    }
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)
        .ok()
        .expect("Couldn't read line");

    println!();
}
