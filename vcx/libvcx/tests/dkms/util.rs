use super::actor::Actor;
use std::env;
use std::thread;
use std::time;

pub fn print_chapter(chap_name: &str, line_len: Option<usize>) {
    let line_len = line_len.unwrap_or(64);
    let short_star_len = 5;
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
                _ => panic!("Actor '{}' is not a valid option", actor)
            }
        },
        Err(_) => {
            panic!("Actor was not specified!")
        },
    }
}

#[allow(dead_code)]
pub fn long_sleep() {
    let ten_s = time::Duration::from_secs(25);
    thread::sleep(ten_s);
}