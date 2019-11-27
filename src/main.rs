#[macro_use]
extern crate arrayref;
#[macro_use]
extern crate bitflags;
extern crate blip_buf;
extern crate clap;
extern crate sdl2;
extern crate time;

mod cpu;
mod debugger;
mod gebemula;
mod graphics;
mod mem;
mod peripherals;
mod util;

use clap::{App, Arg};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::gebemula::Gebemula;

fn main() {
    let args = App::new("Gebemula")
        .author(
            "Bruno Romero de Azevedo <brunordea@gmail.com\n\
             Hugo Stefan Kaus Puhlmann <hugopuhlmann@gmail.com>\n\
             Vitor da Silva <vitords.42@gmail.com>\n\
             Yuri Kunde Schlesner <yuriks@yuriks.net>",
        )
        .about("Emulator for GameBoy written in Rust.")
        .arg(
            Arg::with_name("INPUT_ROM")
                .index(1)
                .required(true)
                .help("Path to the game ROM."),
        )
        .arg(
            Arg::with_name("bootstrap_rom")
                .short("b")
                .long("bootstrap")
                .help("Sets the path to the Gameboy bootstrap ROM.")
                .value_name("DMG_ROM.bin")
                .takes_value(true),
        )
        .get_matches();

    let rom_path = Path::new(args.value_of("INPUT_ROM").unwrap());
    let bootstrap_path = match args.value_of("bootstrap_rom") {
        Some(boot_rom) => Path::new(boot_rom),
        None => {
            if rom_path.extension().unwrap() == "gb" {
                Path::new("DMG_ROM.bin")
            } else {
                Path::new("CGB_ROM.bin")
            }
        }
    };
    // battery files should start with '.'.
    let rom_file_name = rom_path.file_name().unwrap().to_str().unwrap();
    let battery_file_name = &format!(".{}", rom_file_name);
    let battery_path = rom_path
        .with_file_name(battery_file_name)
        .with_extension("sav");

    let mut bootstrap_data = Vec::new();
    File::open(bootstrap_path)
        .expect("Unable to open Bootstrap Rom")
        .read_to_end(&mut bootstrap_data)
        .unwrap();

    let mut game_data = Vec::new();
    File::open(rom_path)
        .expect("Unable to open Game Rom")
        .read_to_end(&mut game_data)
        .unwrap();

    let mut battery_data = Vec::new();
    if battery_path.exists() {
        println!("Loaded battery: {}", battery_path.display());
        File::open(&battery_path)
            .expect("Unable to open Save file")
            .read_to_end(&mut battery_data)
            .unwrap();
    }

    let save_battery_callback = |data: &[u8]| {
        File::create(&battery_path)
            .unwrap()
            .write_all(data)
            .unwrap();
        // Some games use SRAM as non-save scratch space, so this tends to get a bit spammy:
        //println!("Saved battery: {}", battery_path.display());
    };

    // This variable needs to be boxed since it's large and causes a stack overflow in Windows
    let mut gebemula = Box::new(Gebemula::default());
    gebemula.set_save_battery_callback(&save_battery_callback);
    gebemula.load_bootstrap_rom(&bootstrap_data);
    gebemula.load_cartridge(&game_data, &battery_data);
    gebemula.run_sdl();
}
