use std::{net::UdpSocket};
use clap::{Parser, Subcommand};
use std::fmt;
use std::{time::{Duration, Instant}};
pub mod message;
use message::{FacilityRecord, RequestType, QueryRequest, QueryResponse, Booking, BookingResponse, Update, UpdateResponse, Day};

#[derive(Parser, Debug)]
#[command(name = "Facility CLI", about = "A UDP client for facility booking system")]
struct Cli {
    #[arg(short, long, default_value = "127.0.0.1:5000")]
    server: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Query facility availability
    Query {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        days: Vec<String>,
    },

    /// Book a facility
    Book {
        #[arg(long)]
        name: String,
        #[arg(short, long)]
        day: String,
        #[arg(short, long)]
        start_slot: u8,
        #[arg(long)]
        num_slots: u8,
        #[arg(short, long)]
        user_id: u8,
    },

    /// Update booking
    Update {
        #[arg(short, long)]
        confirmation_id: u8,
        #[arg(short, long)]
        offset: i8,
    },

    Monitor {
        #[arg(short, long)]
        duration: u32,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record(pub [u8; 16]);

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Time slots (8:00 AM - 4:00 PM):")?;
        for (i, &slot) in self.0.iter().enumerate() {
            let hour = 8 + i / 2;
            let minute = if i % 2 == 0 { "00" } else { "30" };
            let status = if slot == 0 {
                "Available".to_string()
            } else {
                format!("Booked by {}", slot)
            };
            writeln!(f, "{:02}:{:<2} - {}", hour, minute, status)?;
        }
        Ok(())
    }
}

pub struct Monitor {
    pub duration: u32
}

impl Monitor {
    pub fn serialize(&self, output_stream: &mut Vec<u8>) {
        output_stream.append(self.duration.to_le_bytes().to_vec().as_mut());
    }

    pub fn deserialize(input_stream: &[u8], pos: &mut usize) -> Self {
        let duration = u32::from_le_bytes(input_stream[*pos..*pos+4].try_into().unwrap());
        *pos += 4;
        Self { duration: duration }
    }
}

fn main() {
    let cli = Cli::parse();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let server_address = cli.server;

    match cli.command {
        Commands::Query { name, days } => {
            let days: Vec<Day> = days.iter().map(|d| Day::from(d.as_str())).collect();
            let no_of_days = days.len();
            let req = QueryRequest { name, days };
            let mut output_stream = vec![RequestType::QUERY as u8];
            req.serialize(&mut output_stream);
            socket.send_to(&output_stream, &server_address).unwrap();

            let mut buf = [0; 1024];
            let (num_bytes, _) = socket.recv_from(&mut buf).unwrap();
            let mut pos = 0;
            let resp = QueryResponse::deserialize(&buf[..num_bytes], &mut pos);
            let mut records: Vec<Record> = Vec::new();
            for i in 0..no_of_days {
                let record: Record = Record(resp.availaible[i*16..i*16+16].try_into().unwrap());
                records.push(record);
            }
            for record in records {
                println!("{}", record);
            }
        }

        Commands::Book {
            name,
            day,
            start_slot,
            num_slots,
            user_id,
        } => {
            let booking = Booking {
                facility_name: name,
                day: Day::from(day.as_str()),
                start_slot,
                num_slots,
                user_id,
            };
            let mut output_stream = vec![RequestType::BOOK as u8];
            booking.serialize(&mut output_stream);
            socket.send_to(&output_stream, &server_address).unwrap();

            let mut buf = [0; 1024];
            let (num_bytes, _) = socket.recv_from(&mut buf).unwrap();
            let mut pos = 0;
            let resp = BookingResponse::deserialize(&buf[..num_bytes], &mut pos);
            println!("Booking Response: {:?}", resp);
        }

        Commands::Update {
            confirmation_id,
            offset,
        } => {
            let update = Update {
                confirmation_id,
                offset,
            };
            let mut output_stream = vec![RequestType::UPDATE as u8];
            update.serialize(&mut output_stream);
            socket.send_to(&output_stream, &server_address).unwrap();
            let mut buf = [0; 1024];
            let (num_bytes, _) = socket.recv_from(&mut buf).unwrap();
            let mut pos = 0;
            let resp = UpdateResponse::deserialize(&buf[..num_bytes], &mut pos);
            println!("Booking Response: {:?}", resp);
        }

        Commands::Monitor { duration } => {
            let start = Instant::now();
            let timeout = Duration::from_secs(10); // 1 second per recv attempt
            socket.set_read_timeout(Some(timeout)).unwrap();
            let mut output_stream = vec![RequestType::MONITOR as u8];
            let monitor = Monitor { duration };
            monitor.serialize(&mut output_stream);
            socket.send_to(&output_stream, &server_address).unwrap();

            let mut buf = [0u8; 1024];

            println!("Monitoring for {} seconds...", duration);

            while start.elapsed() < Duration::from_secs(duration as u64) {
                match socket.recv_from(&mut buf) {
                    Ok((num_bytes, src_addr)) => {
                        let mut pos = 0;
                        println!("Received {} bytes from {}", num_bytes, src_addr);
                        let facility: FacilityRecord = FacilityRecord::deserialize(&buf[..num_bytes], &mut pos);
                        println!("{}", facility);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No data received within timeout, continue
                        continue;
                    }
                    Err(_e) => {
                        // eprintln!("UDP receive error: {}", e);
                    }
                }
            }

            println!("Monitoring ended after {} seconds.", duration);
        }
    }
}