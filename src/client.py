import socket
import argparse
import time
from message import (
    Day,
    RequestType,
    QueryRequest,
    QueryResponse,
    Booking,
    BookingResponse,
    Update,
    UpdateResponse,
    Monitor,
    FacilityRecord
)

def main():
    parser = argparse.ArgumentParser(description="A UDP client for facility booking system")
    parser.add_argument("-s", "--server", default="127.0.0.1:5000", help="Server address")
    
    subparsers = parser.add_subparsers(dest="command", required=True)

    # Query command
    query_parser = subparsers.add_parser("query", help="Query facility availability")
    query_parser.add_argument("-n", "--name", required=True)
    query_parser.add_argument("-d", "--days", required=True, nargs="+" )

    # Book command
    book_parser = subparsers.add_parser("book", help="Book a facility")
    book_parser.add_argument("--name", required=True)
    book_parser.add_argument("-d", "--day", required=True)
    book_parser.add_argument("-s", "--start-slot", type=int, required=True)
    book_parser.add_argument("--num-slots", type=int, required=True)
    book_parser.add_argument("-u", "--user-id", type=int, required=True)

    # Update command  
    update_parser = subparsers.add_parser("update", help="Update booking")
    update_parser.add_argument("-c", "--confirmation-id", type=int, required=True)
    update_parser.add_argument("-o", "--offset", type=int, required=True)

    # Monitor command
    monitor_parser = subparsers.add_parser("monitor", help="Monitor facility")
    monitor_parser.add_argument("-d", "--duration", type=int, required=True)

    args = parser.parse_args()

    # Setup UDP socket
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    server_host, server_port = args.server.split(":")
    server_addr = (server_host, int(server_port))

    if args.command == "query":
        # Handle query command
        days = [Day.from_string(d) for d in args.days]
        req = QueryRequest(name=args.name, days=days)
        
        output = bytearray([RequestType.QUERY.value])
        output.extend(req.serialize())
        
        sock.sendto(bytes(output), server_addr)
        
        data, _ = sock.recvfrom(1024)
        resp = QueryResponse.deserialize(data)
        
        for i in range(len(days)):
            day_slots = resp.available[i*16:(i+1)*16]
            print(f"\n{days[i].name}:")
            print_slots(day_slots)

    elif args.command == "book":
        # Handle book command
        booking = Booking(
            facility_name=args.name,
            day=Day.from_string(args.day),
            start_slot=args.start_slot,
            num_slots=args.num_slots,
            user_id=args.user_id
        )
        
        output = bytearray([RequestType.BOOK.value])
        output.extend(booking.serialize())
        
        sock.sendto(bytes(output), server_addr)
        
        data, _ = sock.recvfrom(1024)
        resp = BookingResponse.deserialize(data)
        print(f"Booking Response: {resp}")

    elif args.command == "monitor":
        # Handle monitor command
        sock.settimeout(10.0)
        
        monitor = Monitor(duration=args.duration)
        output = bytearray([RequestType.MONITOR.value])
        output.extend(monitor.serialize())
        
        sock.sendto(bytes(output), server_addr)
        
        start_time = time.time()
        print(f"Monitoring for {args.duration} seconds...")
        
        while time.time() - start_time < args.duration:
            try:
                data, _ = sock.recvfrom(1024)
                facility = FacilityRecord.deserialize(data)
                print(str(facility))
            except socket.timeout:
                continue
            except Exception as e:
                print(f"Error: {e}")
                
        print(f"Monitoring ended after {args.duration} seconds.")

def print_slots(slots):
    print("Time slots (8:00 AM - 4:00 PM):")
    for i, slot in enumerate(slots):
        hour = 8 + i // 2
        minute = "00" if i % 2 == 0 else "30"
        status = "Available" if slot == 0 else f"Booked by {slot}"
        print(f"{hour:02d}:{minute} - {status}")

if __name__ == "__main__":
    main()