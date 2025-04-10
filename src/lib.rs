use std::{cmp, env, thread};
use std::fs::File;
use std::io::{self, Read, Write};
use std::time::Instant;
use std::time::Duration;
extern crate termion;
use termion::{clear, cursor};
use termion::raw;
use termion::raw::IntoRawMode;
use termion::input::TermRead;

fn variable_summary<W: Write + std::os::fd::AsFd>(stdout: &mut raw::RawTerminal<W>, vname: &str, data: Vec<f64>) {
    let (avg, dev) = variable_summary_stats(data);
    variable_summary_print(stdout, vname, avg, dev);
}

fn variable_summary_stats(data: Vec<f64>) -> (f64, f64)
{
    //beregn statistikk
    let N = data.len();
    let sum: f64 = data.iter().sum();
    let avg = sum / (N as f64);
    let dev = (
        data.clone().into_iter()
            .map(|v| (v - avg).powi(2))
            .fold(0.0, |a, b| a+b)
            / (N as f64)
    ).sqrt();
    (avg, dev)
}

fn variable_summary_print<W: Write + std::os::fd::AsFd>(stdout: &mut raw::RawTerminal<W>, vname: &str, avg: f64, dev: f64) where W: std::os::fd::AsFd

{
    //skriv ut formatert utdata
    write!(stdout, "Gjennomsnitt av {:25}{:.6}\r\n", vname, avg);
    write!(stdout, "Standardavvik på {:14}{:.6}\r\n", vname, dev);
    write!(stdout, "\r\n");
}

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
    let buffer = match env::args().nth(1) {
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
    let termwidth = termsize.map(|(w,_)| w - 2).expect("termwidth") as u64;
    let termheight = termsize.map(|(h,_)| h - 2).expect("termheight") as u64;
    let mut _stdout = io::stdout(); //lås én gang, i stedet for én gang per skriving
    let mut stdout = _stdout.lock().into_raw_mode().unwrap();
    let mut record_location = Vec::new();
    let mut record_speed = Vec::new();
    let mut record_acceleration = Vec::new();
    let mut record_voltage = Vec::new();


    while !floor_requests.is_empty() {      
        //5.1. Oppdater plassering, hastighet og akselerasjon
        let now = Instant::now();
        let dt = now.duration_since(perv_loop_time)
            .as_secs_f64();
        perv_loop_time = now;

        record_location.push(location);
        record_speed.push(speed);
        record_acceleration.push(acceleration);
        record_voltage.push(motor_voltage_up - motor_voltage_down);

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
        print!("{}{}{}", clear::All, cursor::Goto(1, 1), cursor::Hide);
        let carriage_floor = (location / floor_height).floor();
        let carriage_floor = if carriage_floor < 1.0 {
            0
        } else {
            carriage_floor as u64
        };
        let carriage_floor = cmp::min(carriage_floor, floor_count - 1);
        let mut terminal_buffer = vec![' ' as u8; (termwidth * termheight) as usize];

        for ty in 0..floor_count {
            terminal_buffer[(ty * termwidth + 0) as usize] = '[' as u8;
            terminal_buffer[(ty * termwidth + 1) as usize] =
                if (ty as u64) == ((floor_count - 1) - carriage_floor) {
                    'X' as u8
                } else {
                    ' ' as u8
                };
            terminal_buffer[(ty * termwidth + 2) as usize] = ']' as u8;
            terminal_buffer[(ty * termwidth + termwidth - 2) as usize] = '\r' as u8;
            terminal_buffer[(ty * termwidth + termwidth - 1) as usize] = '\n' as u8;
        }
        let stats = vec![
            format!("Heisen er i etasje {}", carriage_floor + 1),
            format!("Lokasjon           {:.06}", location),
            format!("Hastighet          {:.06}", speed),
            format!("Akselerasjon       {:.06}", acceleration),
            format!("Spenning [OPP-NED] {:.06}", motor_voltage_up - motor_voltage_down),
        ];
        for sy in 0..stats.len() {
            for (sx,sc) in stats[sy].chars().enumerate() {
                terminal_buffer[sy * (termwidth as usize) + 6 + sx] = sc as u8;
            }
        }
        write!(stdout, "{}", String::from_utf8(terminal_buffer).unwrap());
        stdout.flush().unwrap();

    }   thread::sleep(Duration::from_millis(10));
        
        //6. Skriv ut sammendrag   
    write!(stdout, "{}{}{}", clear::All, cursor::Goto(1, 1), cursor::Show).unwrap();
    variable_summary(&mut stdout, "lokasjon", record_location);
    variable_summary(&mut stdout, "hastighet", record_speed);
    variable_summary(&mut stdout, "akselerasjon", record_acceleration);
    variable_summary(&mut stdout, "spenning", record_voltage);
    stdout.flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_stats() {
        let test_data = vec![
            (vec![1.0, 2.0, 3.0, 4.0, 5.0], 3.0, 1.41),
            (vec![1.0, 3.0, 5.0, 7.0, 9.0], 5.0, 2.83),
            (vec![1.0, 9.0, 1.0, 9.0, 1.0], 4.2, 3.92),
            (vec![1.0, 0.5, 0.7, 0.9, 0.6], 0.74, 0.19),
            (vec![200.0, 3.0, 24.0, 92.0, 111.0], 86.0, 69.84),
        ];
        for (data, avg, dev) in test_data
        {
            let (ravg, rdev) = variable_summary_stats(data);
            //det er ikke trygt å bruke direkte == operator på flytere
            //floats kan være *veldig* nærme og ikke lik
            //så i stedet sjekker vi at de er veldig nære i verdi
            assert!( (avg-ravg).abs() < 0.1 );
            assert!( (dev-rdev).abs() < 0.1 );
        }
    }
}
