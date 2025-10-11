use std::{collections::HashMap, net::{SocketAddr, UdpSocket}, thread, time::Duration};
pub mod message;
use message::{FacilityRecord, RequestType, QueryRequest, QueryResponse, Booking, BookingResponse, Update, UpdateResponse, Monitor};

fn main() {

    let mut all_facilities: HashMap<String, FacilityRecord> = HashMap::new();
    all_facilities.insert(String::from("facility1"), FacilityRecord::new());
    all_facilities.insert(String::from("facility2"), FacilityRecord::new());
    all_facilities.insert(String::from("facility3"), FacilityRecord::new());
    let socket: UdpSocket = UdpSocket::bind("127.0.0.1:5000").unwrap();
    let mut booking_counter: u8 = 0;
    let mut booking_list:HashMap<u8, Booking> = HashMap::new();

    let mut monitoring_clients: Vec<SocketAddr> = Vec::new();

    println!("server running on 5000");
    let mut buf:[u8;1024] = [0; 1024];

    loop {
        let bytes: usize;
        let addr: SocketAddr;
        match socket.recv_from(&mut buf) {
            Ok((num_bytes, src_addr)) => {
                // Success: data received
                bytes = num_bytes;
                addr = src_addr;
            }
            Err(e) => {
                // Other I/O error
                eprintln!("An I/O error occurred: {}", e.kind());
                continue;
            }
        }

        let mut pos = 0;
        let request_type: RequestType = RequestType::from(buf[pos]);
        pos = pos + 1;
        match request_type {
            RequestType::QUERY => {
                let req1: QueryRequest = QueryRequest::deserialize(&buf[..bytes], &mut pos);
                let mut availaiblilty: Vec<u8> = Vec::new();
                let facility = all_facilities[&req1.name]; // handle error
                for day in req1.days {
                    availaiblilty.append(&mut facility[day].to_vec());
                }
                let mut query_response: QueryResponse = QueryResponse { name: req1.name, availaible: availaiblilty };
                let mut output_stream: Vec<u8> = Vec::new();
                query_response.serialize(&mut output_stream);
                thread::sleep(Duration::from_secs(10));
                socket.send_to(&output_stream, addr).unwrap();
            }

            RequestType::BOOK => {
                let booking: Booking = Booking::deserialize(&buf[..bytes], &mut pos);
                let facility = all_facilities.get_mut(&booking.facility_name);
                match facility {
                    Some(record) => {
                        let booking_status = record.is_slot_availaible(booking.day, booking.start_slot, booking.num_slots, booking.user_id);
                        match booking_status {
                            true => {
                                booking_counter += 1;
                                booking_list.insert(booking_counter, booking);
                                let booking_response: BookingResponse = BookingResponse { success: true, message: "Booking Successful".to_string(), confirmation_id: booking_counter };
                                let mut output_stream: Vec<u8> = Vec::new();
                                booking_response.serialize(&mut output_stream);
                                thread::sleep(Duration::from_secs(10));
                                socket.send_to(&output_stream, addr).unwrap();

                                output_stream = Vec::new();
                                record.serialize(&mut output_stream);
                                for addr in &monitoring_clients {
                                    socket.send_to(&output_stream, addr).unwrap();
                                }
                            },
                            false => {
                                println!("error in booking already booked");
                                let booking_response: BookingResponse = BookingResponse { success: false, message: "Booking Failed, Slots not availaible".to_string(), confirmation_id: 0 };
                                let mut output_stream: Vec<u8> = Vec::new();
                                booking_response.serialize(&mut output_stream);
                                socket.send_to(&output_stream, addr).unwrap();
                            }
                        }
                    },
                    None => {
                        // return error message
                         let booking_response: BookingResponse = BookingResponse { success: false, message: "Booking Failed, Facility not availaible".to_string(), confirmation_id: 0 };
                        let mut output_stream: Vec<u8> = Vec::new();
                        booking_response.serialize(&mut output_stream);
                        socket.send_to(&output_stream, addr).unwrap();
                    }
                }

            }
            RequestType::UPDATE => {
                let update_request: Update = Update::deserialize(&buf[..bytes], &mut pos);
                let booking: Option<&mut Booking> = booking_list.get_mut(&update_request.confirmation_id);
                match booking {
                    Some(booking) => {
                        let facility = all_facilities.get_mut(&booking.facility_name).unwrap();
                        let update_status: bool = facility.update_booking(booking.day, booking.start_slot, booking.num_slots, booking.user_id, update_request.offset);
                        match update_status {
                            true => {
                                let new_start = booking.start_slot as i8 + update_request.offset;

                                booking.start_slot = new_start as u8; // check for subtraction
                                let update_response: UpdateResponse = UpdateResponse { success: true, message: "Booking updated".to_string() };
                                let mut output_stream: Vec<u8> = Vec::new();
                                update_response.serialize(&mut output_stream);
                                thread::sleep(Duration::from_secs(10));
                                socket.send_to(&output_stream, addr).unwrap();

                            },
                            false => {
                                //return error
                                let update_response: UpdateResponse = UpdateResponse { success: false, message: "Update Failed, Slot not availaible".to_string() };
                                let mut output_stream: Vec<u8> = Vec::new();
                                update_response.serialize(&mut output_stream);
                                socket.send_to(&output_stream, addr).unwrap();
                            }
                        }
                    },
                    None => {
                        // send error
                        let update_response: UpdateResponse = UpdateResponse { success: false, message: "Update Failed, no such booking made".to_string() };
                        let mut output_stream: Vec<u8> = Vec::new();
                        update_response.serialize(&mut output_stream);
                        socket.send_to(&output_stream, addr).unwrap();
                    }
                }
            }
            RequestType::MONITOR => {
                let monitor_request: Monitor = Monitor::deserialize(&buf[..bytes], &mut pos);
                monitoring_clients.push(addr);
                println!("monitoring {:?} for duration {}", addr, monitor_request.duration);
            }
        }
    }

}