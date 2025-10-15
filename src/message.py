
from enum import Enum
from dataclasses import dataclass
from typing import List
import struct



def write_u8(buf: bytearray, v: int) -> None:
    buf.append(v & 0xFF)

def write_i8(buf: bytearray, v: int) -> None:
    buf.append((v + 256) % 256)

def write_u32_le(buf: bytearray, v: int) -> None:
    buf += struct.pack("<I", v)

def write_string(buf: bytearray, s: str) -> None:
    buf += s.encode("utf-8") + b"\x00"


def read_u8(b: bytes, pos: int) -> (int, int):
    return b[pos], pos + 1

def read_i8(b: bytes, pos: int) -> (int, int):
    v, pos = read_u8(b, pos)
    return (v if v < 128 else v - 256), pos

def read_u32_le(b: bytes, pos: int) -> (int, int):
    return struct.unpack_from("<I", b, pos)[0], pos + 4

def read_string(b: bytes, pos: int) -> (str, int):
    start = pos

    while pos < len(b) and b[pos] != 0:
        pos += 1
    if pos >= len(b):
        raise ValueError("Missing null terminator while reading string")
    s = b[start:pos].decode("utf-8")
    pos += 1  
    return s, pos


class RequestType(Enum):
    QUERY = 0
    BOOK = 1
    UPDATE = 2
    MONITOR = 3

class Day(Enum):
    Monday = 0
    Tuesday = 1
    Wednesday = 2
    Thursday = 3
    Friday = 4
    Saturday = 5
    Sunday = 6

    @classmethod
    def from_string(cls, s: str) -> "Day":
        m = {
            "monday": cls.Monday, "tuesday": cls.Tuesday, "wednesday": cls.Wednesday,
            "thursday": cls.Thursday, "friday": cls.Friday, "saturday": cls.Saturday,
            "sunday": cls.Sunday
        }
        k = s.strip().lower()
        if k not in m:
            raise ValueError(f"Invalid day: {s}")
        return m[k]

    def __str__(self) -> str:
        return self.name


@dataclass
class QueryRequest:
    name: str
    days: List[Day]

    def serialize(self) -> bytes:
        buf = bytearray()
        write_string(buf, self.name)
        # write_u8(buf, len(self.days))
        for d in self.days:
            write_u8(buf, d.value)
        return bytes(buf)

@dataclass
class QueryResponse:
  
    available: bytes


    @classmethod
    def deserialize(cls, b: bytes, pos: int = 0) -> "QueryResponse":
        _name, pos = read_string(b, pos)   
        return cls(available=b[pos:])


# ----- 预约 -----
@dataclass
class Booking:
    facility_name: str
    day: Day
    start_slot: int
    num_slots: int
    user_id: int

    def serialize(self) -> bytes:
        buf = bytearray()
        write_string(buf, self.facility_name)
        write_u8(buf, self.day.value)
        write_u8(buf, self.start_slot)
        write_u8(buf, self.num_slots)
        write_u8(buf, self.user_id)
        return bytes(buf)

@dataclass
class BookingResponse:
    confirmation_id: int
    message: str

    @classmethod
    def deserialize(cls, b: bytes, pos: int = 0) -> "BookingResponse":
        cid, pos = read_u32_le(b, pos)
        msg, pos = read_string(b, pos)
        return cls(confirmation_id=cid, message=msg)


@dataclass
class Update:
    confirmation_id: int
    offset: int  
    def serialize(self) -> bytes:
        buf = bytearray()
        write_u32_le(buf, self.confirmation_id)
        write_i8(buf, self.offset)
        return bytes(buf)

# @dataclass
# class UpdateResponse:
#     status: int  

#     @classmethod
#     def deserialize(cls, b: bytes, pos: int = 0) -> "UpdateResponse":
#         status, pos = read_u8(b, pos)
#         msg, pos = read_string(b, pos)
#         return cls(status=status, message=msg)
@dataclass
class UpdateResponse:
    status: int   # 0 success, 1 failure
    message: str

    @classmethod
    def deserialize(cls, b: bytes, pos: int = 0) -> "UpdateResponse":
        status, pos = read_u8(b, pos)
        msg, pos = read_string(b, pos)
        return cls(status=status, message=msg)

@dataclass
class Monitor:
    duration: int  

    def serialize(self) -> bytes:
        buf = bytearray()
        write_u32_le(buf, self.duration)
        return bytes(buf)


@dataclass
class Record:
    slots: bytes 

@dataclass
class FacilityRecord:
    """
    Represents the full weekly schedule for a facility.
    The server sends the complete record (all days) upon any change.
    """
    name: str
    schedule: dict[Day, bytes]  # Maps Day enum to 16-byte slot data

    @classmethod
    def deserialize(cls, b: bytes, pos: int = 0) -> "FacilityRecord":
        # The server sends: facility_name (string) + full record (5 days * 16 bytes)
        name, pos = read_string(b, pos)

        schedule = {}
        # The Rust server sends Monday through Friday
        days_in_order = [Day.Monday, Day.Tuesday, Day.Wednesday, Day.Thursday, Day.Friday]
        
        for day in days_in_order:
            num_slots = 16
            if pos + num_slots > len(b):
                raise ValueError(f"Buffer overflow while reading schedule for {day.name}")
            schedule[day] = b[pos:pos+num_slots]
            pos += num_slots
            
        return cls(name=name, schedule=schedule)

    def __str__(self) -> str:
        lines = [f"--- Facility Update: {self.name} ---"]
        for day, slots in self.schedule.items():
            lines.append(f"\n{day.name}:")
            for i, v in enumerate(slots):
                hour = 8 + i // 2
                minute = "00" if i % 2 == 0 else "30"
                status = "Available" if v == 0 else f"Booked by {v}"
                lines.append(f"  {hour:02d}:{minute} - {status}")
        lines.append("-----------------------")
        return "\n".join(lines)
