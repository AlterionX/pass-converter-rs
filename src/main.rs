mod pass;

use std::{fs::File, io::{Stdin, Write, BufWriter}};

use chrono::Datelike;
use image::Luma;
use pass::{Pass, PkPass, GoogleWalletPass, PassFormat};
use qrcode::QrCode;

use crate::pass::PkPassSubtype;

fn main() {
    println!("Hello, world!");

    let mut found_pkpass = None;
    let mut found_gpass = None;
    let mut mode = "basic".to_owned();
    let mut output = None;
    let mut args_iter = std::env::args();
    // Skip executable name.
    args_iter.next();
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--mode" | "-m" => {
                let Some(path) = args_iter.next() else {
                    println!("Missing mode.");
                    return;
                };

                mode = path;
            },
            "--output" | "-o" => {
                let Some(path) = args_iter.next() else {
                    println!("Missing output.");
                    return;
                };

                output = Some(path);
            },
            "--gpass" | "-g" => {
                let Some(gpass) = args_iter.next() else {
                    println!("Missing file for gpass.");
                    return;
                };

                found_gpass = Some(gpass);
            },
            "--pkpass" | "-p" => {
                let Some(pkpass) = args_iter.next() else {
                    println!("Missing file for pkpass.");
                    return;
                };

                found_pkpass = Some(pkpass);
            },
            _ => {
                println!("Unknown argument found: {arg:?}.");
                return;
            }
        }
    }

    let (pass, target_format) = match (found_pkpass, found_gpass) {
        (Some(pkpass_path), None) => {
            let pkpass = match open_pkpass(pkpass_path.as_str()) {
                Ok(pkpass) => pkpass,
                Err(e) => {
                    println!("Opening and reading provided pkpass at {pkpass_path:?} failed due to {e:?}.");
                    return;
                },
            };

            if let PkPassSubtype::Flight(ref flight) = pkpass.subtype {
                println!("boarding time: {}", flight.parsed_boarding_datetime().unwrap().to_rfc2822());
                println!("departure time: {}", flight.parsed_departure_datetime().unwrap().to_rfc2822());
                let google_json = serde_json::json!({
                    "flightClasses": [{
                        "id": pkpass.base.serial_number, // {issuer}.{id}

                        "origin": {
                          "gate": None::<String>,
                          "airportIataCode": flight.board_point(),
                        },
                        "destination": {
                          "airportIataCode": flight.off_point(),
                        },
                        "flightHeader": {
                          "flightNumber": flight.subsidiary_carrier().map(|s| &s[2..]),
                          "carrier": {
                            "carrierIataCode": flight.subsidiary_carrier().map(|s| &s[..2]),
                            "airlineLogo": None::<String>,
                          },
                        },
                        "localScheduledDepartureDateTime": flight.parsed_departure_datetime().unwrap().to_rfc2822(),
                    }],
                    "flightObjects": [{
                        "id": pkpass.base.serial_number, // {issuer}.{id}
                        "classId": pkpass.base.pass_type_identifier, // {issuer}.{typeid}

                        "barcode": {
                            "type": "AZTEC",
                            "value": pkpass.barcode.message,
                        },

                        "passengerName": flight.passenger(),
                        "boardingAndSeatingInfo": {
                          "seatNumber": flight.seat(),
                          "seatClass": flight.booking_class()
                        },
                        "reservationInfo": {
                          "confirmationCode": flight.recloc()
                        }
                    }]
                });
                println!("gpass: {google_json:#?}");
            }
            // TODO make this pretty
            if mode == "gen" {
                let barcode = &pkpass.barcode;
                let p = output.unwrap_or_else(|| "tmp".to_owned());
                let f_path = if p.ends_with(".png") {
                    p
                } else {
                    format!("{p}.png")
                };
                let data: Vec<_> = barcode.message.as_bytes().iter().copied().collect();

                let code = QrCode::new(data).expect("built");
                let image = code.render::<Luma<u8>>().build();
                image.save(f_path).expect("image save complete");

                todo!("merge this with the proper fork join flow");
            }

            println!("Parsed pkpass: {pkpass:#?}");

            (Pass::from(pkpass), PassFormat::GPass)
        },
        (None, Some(gpass)) => {
            unimplemented!("gpass to pkpass not implemented.");
        },
        (None, None) => {
            println!("No pass provided.");
            return;
        },
        (Some(_pkpass), Some(_gpass)) => {
            println!("Only one of pkpass or gpass should be presented.");
            return;
        },
    };

    let res = match output {
        Some(p) => {
            let f = match File::create(p) {
                Ok(f) => f,
                Err(e) => {
                    println!("Opening file failed due to {e:?}.");
                    return;
                },
            };
            write_pass(f, pass, target_format)
        },
        None => {
            write_pass(std::io::stdout(), pass, target_format)
        },
    };
    let outcome = match res {
        Ok(outcome) => outcome,
        Err(e) => {
            println!("Outputting file format failed due to {e:?}.");
        },
    };

    println!("Success.");
}

fn open_pkpass(path: &str) -> Result<PkPass, anyhow::Error> {
    let f = File::open(path)?;

    let now = chrono::Utc::now();
    let pkpass = PkPass::read(f, now.year())?;

    Ok(pkpass)
}

fn write_pass(output: impl Write, pass: Pass, format: PassFormat) -> Result<(), anyhow::Error> {
    Ok(())
}
