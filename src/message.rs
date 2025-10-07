use std::fmt;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
}

impl From<u8> for Day {
    fn from(item: u8) -> Self {
        match item {
            0 => Day::Monday,
            1 => Day::Tuesday,
            2 => Day::Wednesday,
            3 => Day::Thursday,
            4 => Day::Friday,
            _ => panic!("Invalid day of the week value: {}", item),
        }
    }
}

impl From<&str> for Day {
    fn from(item: &str) -> Self {
        match item.to_lowercase().as_str() {
            "monday" => Day::Monday,
            "tuesday" => Day::Tuesday,
            "wednesday" => Day::Wednesday,
            "thursday" => Day::Thursday,
            "friday" => Day::Friday,
            _ => panic!("Invalid day: {}", item),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Booking {
    pub facility_name: String,
    pub day: Day,
    pub start_slot: u8,
    pub num_slots: u8,
    pub user_id: u8,
}

impl Booking {
    pub fn serialize(&self, output_stream: &mut Vec<u8>) {
        let mut facility_name: Vec<u8> = self.facility_name.as_bytes().to_vec();
        facility_name.push(0);
        output_stream.append(&mut facility_name);
        output_stream.push(self.day as u8);
        output_stream.push(self.start_slot);
        output_stream.push(self.num_slots);
        output_stream.push(self.user_id);
        println!("{:?}", output_stream);
    }
    pub fn deserialize(input_stream: &[u8], pos: &mut usize) -> Self {
        let facility_name = read_string(input_stream, pos);
        let day: Day = Day::from(input_stream[*pos]);
        *pos = *pos + 1;
        let start_slot: u8 = input_stream[*pos];
        *pos = *pos + 1;
        let num_slots: u8 = input_stream[*pos];
        *pos = *pos + 1;
        let user_id: u8 = input_stream[*pos];
        *pos = *pos + 1;
        
        Self { facility_name: facility_name, day: day, start_slot: start_slot, num_slots: num_slots, user_id: user_id }
    }
}


#[derive(Debug)]
pub struct BookingResponse {
    pub success: bool,
    pub message: String,
    pub confirmation_id: u8
}
impl BookingResponse {
    pub fn serialize(&self, output_stream: &mut Vec<u8>) {
        output_stream.push(self.success as u8);
        let mut message_bytes: Vec<u8> = self.message.as_bytes().to_vec();
        message_bytes.push(0);
        output_stream.append(&mut message_bytes);
        output_stream.push(self.confirmation_id);
    }
    pub fn deserialize(input_stream: &[u8], pos: &mut usize) -> Self {
        let success: bool = input_stream[*pos] != 0;
        *pos += 1;
        println!("{:?}", success);
        let message: String = read_string(input_stream, pos);
        let confirmation_id: u8 = input_stream[*pos];
        *pos += 1;
        Self { success: success, message: message, confirmation_id: confirmation_id }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Update {
    pub confirmation_id: u8,
    pub offset: i8
}

impl Update {
    pub fn serialize(&self, output_stream: &mut Vec<u8>) {
        output_stream.push(self.confirmation_id);
        output_stream.push(self.offset as u8);
        println!("{:?}", output_stream);
    }

    pub fn deserialize(input_stream: &[u8], pos: &mut usize) -> Self {
        let confirmation_id: u8 = input_stream[*pos];
        *pos += 1;
        let offset: i8 = input_stream[*pos] as i8;
        *pos += 1;
        Self { confirmation_id: confirmation_id, offset: offset }
    }
}

#[derive(Debug)]
pub struct UpdateResponse {
    pub success: bool,
    pub message: String
}

impl UpdateResponse {
    pub fn serialize(&self, output_stream: &mut Vec<u8>) {
        output_stream.push(self.success as u8);
        let mut message_bytes: Vec<u8> = self.message.as_bytes().to_vec();
        message_bytes.push(0);
        output_stream.append(&mut message_bytes);
    }
    pub fn deserialize(input_stream: &[u8], pos: &mut usize) -> Self {
        let success: bool = input_stream[*pos] != 0;
        *pos += 1;
        let message: String = read_string(input_stream, pos);
        Self { success: success, message: message }
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

#[derive(Debug, Copy, Clone)]
pub enum RequestType {
    QUERY, BOOK, UPDATE, MONITOR
}

impl From<u8> for RequestType {
    fn from(item: u8) -> Self {
        match item {
            0 => RequestType::QUERY,
            1 => RequestType::BOOK,
            2 => RequestType::UPDATE,
            3 => RequestType::MONITOR,
            _ => panic!("Invalid day of the week value: {}", item),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueryRequest {
    pub name: String,
    pub days: Vec<Day>
}
pub fn read_string(input_stream: &[u8], pos: &mut usize) -> String {
    let mut str_vec: Vec<u8> = Vec::new();
    while input_stream[*pos] != 0 {
        str_vec.push(input_stream[*pos]);
        *pos = *pos + 1;
    }
    *pos = *pos + 1;
    let str_name: String = String::from_utf8(str_vec).unwrap();
    str_name
}
impl QueryRequest {
    pub fn serialize(&self, output_stream: &mut Vec<u8>) {
        let mut name_bytes: Vec<u8> = self.name.as_bytes().to_vec();
        name_bytes.push(0);
        output_stream.append(&mut name_bytes);
        for day in &self.days {
            let u8_day = *day as u8;
            output_stream.push(u8_day);
        }
        println!("{:?}", output_stream);
    }

    pub fn deserialize(input_stream: &[u8], pos: &mut usize) -> Self {
        let name = read_string(input_stream, pos);
        let mut days: Vec<Day> = Vec::new();
        let len = input_stream.len();
        for i in *pos..len {
            let day: Day = Day::from(input_stream[i]);
            days.push(day);
        }
        *pos = *pos + len;
        Self { name: name, days: days }
    }
}

#[derive(Debug)]
pub struct QueryResponse {
    pub name: String,
    pub availaible: Vec<u8>
}
impl QueryResponse {
    pub fn serialize(&mut self, output_stream: &mut Vec<u8>) {
        let mut name_bytes: Vec<u8> = self.name.as_bytes().to_vec();
        name_bytes.push(0);
        output_stream.append(&mut name_bytes);
        output_stream.append(&mut self.availaible);
    }
    pub fn deserialize(input_stream: &[u8], pos: &mut usize) -> Self {
        let name = read_string(input_stream, pos);
        let availaible: Vec<u8> = input_stream[*pos..].to_vec();
        Self { name: name, availaible: availaible }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FacilityRecord {
    monday: [u8;16],
    tuesday: [u8;16],
    wednesday: [u8;16],
    thursday: [u8;16],
    friday: [u8;16]
}
impl FacilityRecord {
    pub fn new() -> Self {
        Self { monday: [0;16], tuesday: [0;16], wednesday: [0;16], thursday: [0;16], friday: [0;16] }
    }

    pub fn serialize(&self, output_stream: &mut Vec<u8>) {
        output_stream.append(self.monday.to_vec().as_mut());
        output_stream.append(self.tuesday.to_vec().as_mut());
        output_stream.append(self.wednesday.to_vec().as_mut());
        output_stream.append(self.thursday.to_vec().as_mut());
        output_stream.append(self.friday.to_vec().as_mut());
    }

    pub fn deserialize(input_stream: &[u8], pos: &mut usize) -> Self {
        let monday: [u8; 16] = input_stream[*pos..*pos+16].try_into().unwrap();
        *pos += 16;
        let tuesday: [u8; 16] = input_stream[*pos..*pos+16].try_into().unwrap();
        *pos += 16;
        let wednesday: [u8; 16] = input_stream[*pos..*pos+16].try_into().unwrap();
        *pos += 16;
        let thursday: [u8; 16] = input_stream[*pos..*pos+16].try_into().unwrap();
        *pos += 16;
        let friday: [u8; 16] = input_stream[*pos..*pos+16].try_into().unwrap();
        *pos += 16;
        Self { monday: monday, tuesday: tuesday, wednesday: wednesday, thursday: thursday, friday: friday }
    }

    pub fn is_slot_availaible(&mut self, day: Day, start_slot: u8, num_slot: u8, user_id: u8) -> bool {
        let record = &mut self[day];

        let start = start_slot as usize;
        let end = (start_slot + num_slot) as usize;

        // Bounds check to avoid panic
        if end > record.len() {
            return false;
        }

        // Check if all slots in range are free (0)
        if record[start..end].iter().all(|&x| x == 0) {
            // Mark them as booked by user_id
            for slot in &mut record[start..end] {
                *slot = user_id;
            }
            true
        } else {
            false
        }
    }

    pub fn update_booking(&mut self, day: Day, start_slot: u8, num_slot: u8, user_id: u8, offset: i8) -> bool {
        let record = &mut self[day];
        let start = start_slot as isize;
        let end = start + num_slot as isize;
        let new_start = start + offset as isize;
        let new_end = end + offset as isize;
        let len = record.len() as isize;

        //Bounds check
        if new_start < 0 || new_end > len {
            return false;
        }

        //Check if the existing booking matches user_id
        if record[start as usize..end as usize]
            .iter()
            .any(|&slot| slot != user_id)
        {
            // The current slots are not owned by this user
            return false;
        }

        //Check if new range overlaps with existing non-zero slots
        if record[new_start as usize..new_end as usize]
            .iter()
            .any(|&slot| slot != 0 && slot != user_id)
        {
            return false;
        }

        //Clear old slots
        for slot in &mut record[start as usize..end as usize] {
            *slot = 0;
        }

        //5. Fill new slots
        for slot in &mut record[new_start as usize..new_end as usize] {
            *slot = user_id;
        }
        true
    }
    
    fn get_day_slots(&self, day: Day) -> &[u8;16] {
        match day {
            Day::Monday => &self.monday,
            Day::Tuesday => &self.tuesday,
            Day::Wednesday => &self.wednesday,
            Day::Thursday => &self.thursday,
            Day::Friday => &self.friday,
        }
    }
}

impl Index<Day> for FacilityRecord {
    type Output = [u8; 16];

    fn index(&self, day: Day) -> &Self::Output {
        match day {
            Day::Monday => &self.monday,
            Day::Tuesday => &self.tuesday,
            Day::Wednesday => &self.wednesday,
            Day::Thursday => &self.thursday,
            Day::Friday => &self.friday,
        }
    }
}

impl IndexMut<Day> for FacilityRecord {
    fn index_mut(&mut self, day: Day) -> &mut Self::Output {
        match day {
            Day::Monday => &mut self.monday,
            Day::Tuesday => &mut self.tuesday,
            Day::Wednesday => &mut self.wednesday,
            Day::Thursday => &mut self.thursday,
            Day::Friday => &mut self.friday,
        }
    }
}


impl fmt::Display for FacilityRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "🏢 Facility Weekly Schedule (8:00 AM - 4:00 PM):")?;

        for day in [Day::Monday, Day::Tuesday, Day::Wednesday, Day::Thursday, Day::Friday] {
            let slots = self.get_day_slots(day);
            writeln!(f, "\n📅 {:?}:", day)?;
            for (i, &slot) in slots.iter().enumerate() {
                let hour = 8 + i / 2;
                let minute = if i % 2 == 0 { "00" } else { "30" };
                let status = if slot == 0 {
                    "Available".to_string()
                } else {
                    format!("Booked by {}", slot)
                };
                writeln!(f, "{:02}:{:<2} - {}", hour, minute, status)?;
            }
        }

        Ok(())
    }
}