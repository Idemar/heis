use std::{env, thread};
use std::io;
use std::fs::File;
use std::io::Read;
use std::time::Instant;

pub fn start_simulator() {
    
    // 1. Lagre etasje, hastighet og akselerasjonstilstand
    let mut location: f64 = 0.0; // meter
    let mut speed: f64 = 0.0; // meter per sekund
    let mut acceleration:f64 = 0.0; // meter per sekund^2
                                 
    // 2. Lagre motor inngangsspenning
    let mut motor_voltage_up: f64 = 0.0;
    let mut motor_voltage_down: f64 = 0.0;

    // 3. Lagre inndata bygningsbeskrivelse og etasje ønsker
    let mut floor_count: u64 = 0;
    let mut floor_height: f64 = 0.0; // meter
    let mut floor_requests: Vec<u64> = Vec::new(); // etasje forespørsel

    // 4. Analyser inndata og lagre som bygningsbeskrivelse og etasje ønsker
    let mut buffer = match env::args().nth(1) {
        Some(ref fil) if *fil == "-".to_string() => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)
                .expect("read_to_string feilet");
            buffer
        },
        None => {
            let fil = "test1.txt";
            let mut buffer = String::new();
            File::open(fil)
                .expect("File::open feilet")
                .read_to_string(&mut buffer)
                .expect("read_to_string feilet");
            buffer
        },
        Some(fil) => {
            let mut buffer = String::new();
            File::open(fil)
                .expect("File::open feilet")
                .read_to_string(&mut buffer)
                .expect("read_to_string feilet");
            buffer
        }
    };

    for (li, l) in buffer.lines().enumerate() {
        if li == 0 {
            floor_count = l.parse::<u64>().unwrap();
        } else if li == 1 {
            floor_height = l.parse::<f64>().unwrap();
        } else {
            floor_requests.push(l.parse::<u64>().unwrap());
        }
    }

    //5. Loop mens det er gjenværende etasjen forespørsler 
    let mut perv_loop_time = Instant::now();

    while !floor_requests.is_empty() {      
        //5.1. Oppdater plassering, hastighet og akselerasjon
        let now = Instant::now();
        let dt = now.duration_since(perv_loop_time)
            .as_secs_f64();
        perv_loop_time = now;


        //5.2. Hvis forespørselen om neste etasje i køen er tilfredsstilt, fjern deretter fra køen
        //5.3. Juster motorkontrollen for å behandle forespørselen neste etasje
        //5.4. Skriv ut sanntidsstatistikk
    }   thread::sleep(time::Duration::from_millis(10));
        
        //6. Skriv ut sammendrag   
        println!("sammendrag");
}
