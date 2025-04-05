use std::{env, thread};
use std::io;
use std::fs::File;
use std::io::{stdout, Read};
use std::time::Instant;
use std::time::Duration;

pub fn start_simulator() {
    
    // 1. Lagre etasje, hastighet og akselerasjonstilstand
    let mut location: f64 = 0.0; // meter
    let mut speed: f64 = 0.0; // meter per sekund
    let mut acceleration: f64 = 0.0; // meter per sekund^2
                                 
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

    //5. Loop mens det er gjenværende etasje forespørsler
    let mut perv_loop_time = Instant::now();
    let termsize = termion::terminal_size().ok();
    let termwidth = termsize.map(|(w,_)| w - 2).expect("termwidth");
    let termheight = termsize.map(|(h,_)| h - 2).expect("termheight");
    let mut _stdout = io::stdout(); //lås én gang, i stedet for én gang per skriving
    let mut stdout = _stdout.lock().into_raw_mode().unwrap();


    while !floor_requests.is_empty() {      
        //5.1. Oppdater plassering, hastighet og akselerasjon
        let now = Instant::now();
        let dt = now.duration_since(perv_loop_time)
            .as_secs_f64();
        perv_loop_time = now;

        location = location + speed * dt;
        speed = speed + acceleration * dt;
        acceleration = {
            let F =(motor_voltage_up - motor_voltage_down) * 8.0;
            let m = 1200000.0;
            -9.8 + F/m
        };

        //5.2. Hvis forespørselen om neste etasje i køen er tilfredsstilt, fjern deretter fra køen
        let next_floor = floor_requests[0];
        if (location - (next_floor as f64) * floor_height).abs() < 0.01 &&
            speed.abs() < 0.01 {
            speed = 0.0;
            floor_requests.remove(0);
        }
        //5.3. Juster motorkontrollen for å behandle forespørselen neste etasje
        //det vil ta t sekunder å bremse fra hastigheten v fra -1 m/s^2
        let t = speed.abs() / 1.0;

        //i løpet av denne tiden vil vognen reise d=t * v/2 meter
        //med en gjennomsnittshastighet på v/2 før stopp
        let d = t * (speed / 2.0);

        //l = avstand til neste etasje
        let l = (location - (next_floor as f64) * floor_height).abs();

        let target_acceleration = {
            // skal vi opp?
            let going_up = location < (next_floor as f64) * floor_height;

            //Ikke overskrid maksimal hastighet
            if speed.abs() >= 5.0 {
                //hvis vi skal opp og faktisk går opp
                //eller vi skal ned og faktisk går ned
                if (going_up && speed > 0.0) || (!going_up && speed < 0.0) {
                    0.0
                    //bremse hvis du går i feil retning
                } else if going_up {
                    1.0
                } else {
                    -1.0
                }
            //Hvis du er innenfor behagelig retardasjonsområde og beveger deg i riktig retning, retarder       
            } else if l < d && going_up == (speed > 0.0) {
                if going_up {
                    -1.0
                } else {
                    1.0
                }
            //ellers hvis ikke ved topphastighet, akselerer
            } else {
                if going_up {
                    1.0
                } else {
                    -1.0
                }
            }
        };

        let gravity_adjusted_acceleration = target_acceleration + 9.8;
        let target_force = gravity_adjusted_acceleration * 1200000.0;
        let target_voltage = target_force / 8.0;
        if target_voltage > 0.0 {
            motor_voltage_up = target_voltage;
            motor_voltage_down = 0.0;
        } else {
            motor_voltage_up = 0.0;
            motor_voltage_down = target_voltage.abs();
        };

        //5.4. Skriv ut sanntidsstatistikk
    }   thread::sleep(Duration::from_millis(10));
        
        //6. Skriv ut sammendrag   
        println!("sammendrag");
}
