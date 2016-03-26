#![feature(box_syntax)]
#![feature(plugin)]
#![plugin(clippy)]

// Disable some clippy warnings we don't particularly care about
#![allow(similar_names)]
#![allow(many_single_char_names)]

#[macro_use]
extern crate bitflags;
extern crate sdl2;
extern crate time;

mod graphics;
mod debugger;
mod cpu;
mod mem;
mod util;
mod gebemula;
mod timeline;

use std::env;
use std::io::Read;
use std::fs::File;

use gebemula::Gebemula;

#[allow(boxed_local)]
fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 3 {
        let mut bootstrap_data: Vec<u8> = Vec::new();
        File::open(&args[1]).unwrap().read_to_end(&mut bootstrap_data).unwrap();

        let mut game_data: Vec<u8> = Vec::new();
        File::open(&args[2]).unwrap().read_to_end(&mut game_data).unwrap();

        // This variable needs to be boxed since it's large and causes a stack overflow in Windows
        let mut gebemula = box Gebemula::default();
        gebemula.load_game_rom(&game_data);
        gebemula.load_bootstrap_rom(&bootstrap_data);
        gebemula.run_sdl();
    } else {
        println!("Invalid number of arguments.");
    }
}
