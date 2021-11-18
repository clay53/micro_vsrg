use std::{io::{
    Cursor,
    Write,
}, time::{
    Duration,
    Instant,
}};

use rodio::Source;
use rppal::gpio::{Gpio, Level};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const P1LED1PIN: u8 = 2;
const P1LED2PIN: u8 = 3;
const P1LED3PIN: u8 = 27;
const P1LED4PIN: u8 = 10;

const BUTTON_DEBOUNCING: Duration = Duration::from_millis(50);
const P1B1PIN: u8 = 4;
const P1B2PIN: u8 = 17;
const P1B3PIN: u8 = 22;
const P1B4PIN: u8 = 9;

const HIT_RANGE: isize = 2000;

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    println!("Welcome to Micro VSRG {}", VERSION);

    let gpio = Gpio::new().unwrap();
    let mut p1led1 = gpio.get(P1LED1PIN).unwrap().into_output_low();
    let mut p1led2 = gpio.get(P1LED2PIN).unwrap().into_output_low();
    let mut p1led3 = gpio.get(P1LED3PIN).unwrap().into_output_low();
    let mut p1led4 = gpio.get(P1LED4PIN).unwrap().into_output_low();
    let p1b1 = gpio.get(P1B1PIN).unwrap().into_input_pulldown();
    let p1b2 = gpio.get(P1B2PIN).unwrap().into_input_pulldown();
    let p1b3 = gpio.get(P1B3PIN).unwrap().into_input_pulldown();
    let p1b4 = gpio.get(P1B4PIN).unwrap().into_input_pulldown();

    println!("Loading maps from ./map_depot ...");
    let depot = std::fs::read_dir("./map_depot").unwrap();
    let mut sets = Vec::new();
    for entry in depot {
        let entry = std::fs::File::open(entry.unwrap().path()).unwrap();
        sets.push(micro_vsrg::Set::from_osz(entry));
    }
    println!("{} set(s) loaded!", sets.len());

    println!("\nSet | Map");
    for (set_id, set) in sets.iter().enumerate() {
        println!("{} ~~~~~~~", set_id);
        for (map_id, map) in set.maps.iter().enumerate() {
            println!("    {}: {:#?}", map_id, map.full_title);
        }
    }
    print!("Select a map {{set_id}},{{map_id}}: ");
    stdout.flush().unwrap();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    let parts: Vec<&str> = input.trim().split(",").collect();
    if parts.len() != 2 {
        eprintln!("Incorrect formatting. Example: 0,1");
        return;
    }
    let set = sets.get(parts[0].parse::<usize>().expect("couldn't parse set id")).expect("couldn't get set");
    let map = set.maps.get(parts[1].parse::<usize>().expect("couldn't parse map id")).expect("couldn't get map");

    println!("Starting... {}", map.full_title);

    let audio = set.files.get(&map.audio_file_name).expect("couldn't get map's audio file").clone();
    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    let source = rodio::Decoder::new(Cursor::new(audio)).expect("failed to create decoder");
    sink.pause();
    sink.append(source.delay(std::time::Duration::from_millis(map.audio_lead_in.try_into().unwrap())));

    let mut player_1_notes = [0; 4];
    let mut player_1_hit: usize = 0;
    let mut player_1_missed: usize = 0;
    let mut _player_2_notes = [0; 4];

    // Setup button timers
    let mut last_p1b1_pressed = Instant::now();
    let mut last_p1b2_pressed = last_p1b1_pressed.clone();
    let mut last_p1b3_pressed = last_p1b1_pressed.clone();
    let mut last_p1b4_pressed = last_p1b1_pressed.clone();
    std::thread::sleep(BUTTON_DEBOUNCING); // sleep to ensure instants are back far enough. TODO: figure out how to set instances to long ago

    let timer = std::time::Instant::now();
    sink.play();

    loop {
        let time: isize = timer.elapsed().as_millis().try_into().unwrap();
        let mut done = true;
        while let Some(p1c1) = map.notes[0].get(player_1_notes[0]) {
            done = false;
            let p1c1: isize = (*p1c1).try_into().unwrap();
            let diff = p1c1-time;
            if diff < HIT_RANGE && diff > -HIT_RANGE {
                p1led1.set_high();
                let p1b1_level = p1b1.read();
                if p1b1_level==Level::High {
                    if last_p1b1_pressed.elapsed() >= BUTTON_DEBOUNCING {
                        p1led1.set_low();
                        player_1_hit += 1;
                        player_1_notes[0] += 1;
                        println!("P1B1 PRESSED!");
                    }
                    last_p1b1_pressed = Instant::now();
                }
                break;
            } else if diff < -HIT_RANGE {
                p1led1.set_low();
                player_1_missed += 1;
                println!("P1B1 MISSED!");
                player_1_notes[0] += 1;
            } else {
                break;
            }
        }

        while let Some(p1c2) = map.notes[1].get(player_1_notes[1]) {
            done = false;
            let p1c2: isize = (*p1c2).try_into().unwrap();
            let diff = p1c2-time;
            if diff < HIT_RANGE && diff > -HIT_RANGE {
                p1led2.set_high();
                let p1b2_level = p1b2.read();
                if p1b2_level==Level::High {
                    if last_p1b2_pressed.elapsed() >= BUTTON_DEBOUNCING {
                        p1led2.set_low();
                        player_1_hit += 1;
                        player_1_notes[1] += 1;
                        println!("P1B2 PRESSED!");
                    }
                    last_p1b2_pressed = Instant::now();
                }
                break;
            } else if diff < -HIT_RANGE {
                p1led2.set_low();
                player_1_missed += 1;
                println!("P1B2 MISSED!");
                player_1_notes[1] += 1;
            } else {
                break;
            }
        }

        while let Some(p1c3) = map.notes[2].get(player_1_notes[2]) {
            done = false;
            let p1c3: isize = (*p1c3).try_into().unwrap();
            let diff = p1c3-time;
            if diff < HIT_RANGE && diff > -HIT_RANGE {
                p1led3.set_high();
                let p1b3_level = p1b3.read();
                if p1b3_level==Level::High {
                    if last_p1b3_pressed.elapsed() >= BUTTON_DEBOUNCING {
                        p1led3.set_low();
                        player_1_hit += 1;
                        player_1_notes[2] += 1;
                        println!("P1B3 PRESSED!");
                    }
                    last_p1b3_pressed = Instant::now();
                }
                break;
            } else if diff < -HIT_RANGE {
                p1led3.set_low();
                player_1_missed += 1;
                println!("P1B3 MISSED!");
                player_1_notes[2] += 1;
            } else {
                break;
            }
        }

        while let Some(p1c4) = map.notes[3].get(player_1_notes[3]) {
            done = false;
            let p1c4: isize = (*p1c4).try_into().unwrap();
            let diff = p1c4-time;
            if diff < HIT_RANGE && diff > -HIT_RANGE {
                p1led4.set_high();
                let p1b4_level = p1b4.read();
                if p1b4_level==Level::High {
                    if last_p1b4_pressed.elapsed() >= BUTTON_DEBOUNCING {
                        p1led4.set_low();
                        player_1_hit += 1;
                        player_1_notes[3] += 1;
                        println!("P1B4 PRESSED!");
                    }
                    last_p1b4_pressed = Instant::now();
                }
                break;
            } else if diff < -HIT_RANGE {
                p1led4.set_low();
                player_1_missed += 1;
                println!("P1B4 MISSED!");
                player_1_notes[3] += 1;
            } else {
                break;
            }
        }

        if done {
            break;
        } 
    }

    println!("Accuracy: {}/{}", player_1_hit, player_1_hit+&player_1_missed);
}