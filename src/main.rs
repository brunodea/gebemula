#![feature(box_syntax)]
// This is soon going to be stabilized in 1.9.0
#![feature(copy_from_slice)]
#![feature(plugin)]
#![plugin(clippy)]

// Disable some clippy warnings we don't particularly care about
#![allow(similar_names)]
#![allow(many_single_char_names)]
#![allow(if_not_else)]

#[macro_use]
extern crate bitflags;
extern crate clap;
extern crate sdl2;
extern crate time;

mod graphics;
mod debugger;
mod cpu;
mod mem;
mod util;
mod gebemula;
mod timeline;

use clap::{Arg, App};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use gebemula::Gebemula;

#[allow(boxed_local)]
fn main() {
    let args = App::new("Gebemula")
            .author("Bruno Romero de Azevedo <brunodea@inf.ufsm.br>")
            .about("Emulator for GameBoy written in Rust.")
            .arg(Arg::with_name("INPUT_ROM")
                 .index(1).required(true)
                 .help("Path to the game ROM."))
            .arg(Arg::with_name("bootstrap_rom")
                 .short("b").long("bootstrap")
                 .help("Sets the path to the Gameboy bootstrap ROM.")
                 .value_name("DMG_ROM.bin")
                 .takes_value(true)
                 .default_value("DMG_ROM.bin"))
            .get_matches();

    let bootstrap_path = Path::new(args.value_of("bootstrap_rom").unwrap());
    let rom_path = Path::new(args.value_of("INPUT_ROM").unwrap());
    let battery_path = rom_path.with_extension("sav");

    let mut bootstrap_data = Vec::new();
    File::open(bootstrap_path).unwrap().read_to_end(&mut bootstrap_data).unwrap();

    let mut game_data = Vec::new();
    File::open(rom_path).unwrap().read_to_end(&mut game_data).unwrap();

    let mut battery_data = Vec::new();
    if battery_path.exists() {
        println!("Loaded battery: {}", battery_path.display());
        File::open(&battery_path).unwrap().read_to_end(&mut battery_data).unwrap();
    }

    let save_battery_callback = |data: &[u8]| {
        File::create(&battery_path).unwrap().write_all(data).unwrap();
        // Some games use SRAM as non-save scratch space, so this tends to get a bit spammy:
        //println!("Saved battery: {}", battery_path.display());
    };

    // This variable needs to be boxed since it's large and causes a stack overflow in Windows
    let mut gebemula = box Gebemula::default();
    gebemula.set_save_battery_callback(&save_battery_callback);
    gebemula.load_bootstrap_rom(&bootstrap_data);
    gebemula.load_cartridge(&game_data, &battery_data);
    gebemula.run_sdl();
}
