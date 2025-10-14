# src/client.py
import argparse
import socket
import time

from message import (
    RequestType, Day,
    QueryRequest, QueryResponse,
    Booking, BookingResponse,
    Update, UpdateResponse,
    Monitor, FacilityRecord
)


def print_slots(day_slots: bytes) -> None:
    # 槽 0..15 映射 08:00..15:30（每 30min）
    for i, v in enumerate(day_slots):
        hour = 8 + i // 2
        minute = "00" if i % 2 == 0 else "30"
        status = "Available" if v == 0 else f"Booked by {v}"
        print(f"{hour:02d}:{minute} - {status}")


_next_req_id = -1
def next_req_id() -> int:
    global _next_req_id
    _next_req_id = (_next_req_id + 1) & 0xFF
    return _next_req_id

# ----------- 统一发送/接收（含超时与重试）-----------
def send_and_recv(sock: socket.socket,
                  server_addr,
                  out_bytes: bytes,
                  timeout_s: float = 3.0,
                  retries: int = 2,
                  semantics: str = "alo") -> bytes:
    """
    semantics:
      - "alo": at-least-once（超时就重发，共 retries+1 次机会）
      - "amo": at-most-once（只发一次，超时不重发）
    """
    sock.settimeout(timeout_s)
    attempts = 1 if semantics == "amo" else (retries + 1)
    last_err = None
    for _ in range(attempts):
        try:
            sock.sendto(out_bytes, server_addr)
            data, _ = sock.recvfrom(4096)
            return data
        except Exception as e:
            last_err = e
            continue
    if last_err:
        raise last_err
    raise RuntimeError("send_and_recv: unexpected empty last_err")

def main():
    parser = argparse.ArgumentParser(description="UDP Client for Facility Reservation")

    # 顶层实验参数（便于课程实验）
    parser.add_argument("--timeout", type=float, default=3.0, help="recv timeout seconds")
    parser.add_argument("--retries", type=int, default=2, help="retries for at-least-once")
    parser.add_argument("--semantics", choices=["alo", "amo"], default="alo",
                        help="invocation semantics: alo=at-least-once, amo=at-most-once")

    parser.add_argument("-s", "--server", required=True,
                        help="server in form ip:port, e.g. 127.0.0.1:5000")
    subparsers = parser.add_subparsers(dest="command", required=True)

    # query
    p_query = subparsers.add_parser("query", help="query availability")
    p_query.add_argument("-n", "--name", required=True, help="facility name")
    p_query.add_argument("-d", "--days", required=True, nargs="+",
                         help="days list, e.g. monday tuesday")

    # book
    p_book = subparsers.add_parser("book", help="book a facility")
    p_book.add_argument("-n", "--name", required=True, help="facility name")
    p_book.add_argument("-d", "--day", required=True, help="day, e.g. monday")
    p_book.add_argument("-s", "--start-slot", required=True, type=int, help="start slot (0..15)")
    p_book.add_argument("--num-slots", required=True, type=int, help="number of slots")
    p_book.add_argument("-u", "--user-id", required=True, type=int, help="user id (0..255)")

    # update
    p_update = subparsers.add_parser("update", help="update an existing booking by confirmation id and offset")
    p_update.add_argument("-c", "--confirmation-id", required=True, type=int, help="confirmation id")
    p_update.add_argument("-o", "--offset", required=True, type=int, help="offset slots (can be negative)")

    # monitor
    p_monitor = subparsers.add_parser("monitor", help="monitor facility updates for duration seconds")
    p_monitor.add_argument("-d", "--duration", required=True, type=int, help="duration seconds")

    args = parser.parse_args()

    # 解析 server 地址
    try:
        host, port_s = args.server.split(":")
        server_addr = (host, int(port_s))
    except Exception:
        raise ValueError("Invalid --server, expected ip:port, e.g. 127.0.0.1:5000")

    # UDP socket
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

    if args.command == "query":
        days = [Day.from_string(d) for d in args.days]
        req = QueryRequest(name=args.name, days=days)

        rid = next_req_id()
        output = bytearray([RequestType.QUERY.value, rid])
        output.extend(req.serialize())

        data = send_and_recv(sock, server_addr, bytes(output),
                             timeout_s=args.timeout, retries=args.retries, semantics=args.semantics)

        # 跳过 [resp_type][resp_id]
        pos = 2
        resp = QueryResponse.deserialize(data, pos)

        for i in range(len(days)):
            day_slots = resp.available[i*16:(i+1)*16]
            print(f"\n{days[i].name}:")
            print_slots(day_slots)

    elif args.command == "book":
        booking = Booking(
            facility_name=args.name,
            day=Day.from_string(args.day),
            start_slot=args.start_slot,
            num_slots=args.num_slots,
            user_id=args.user_id
        )

        rid = next_req_id()
        output = bytearray([RequestType.BOOK.value, rid])
        output.extend(booking.serialize())

        data = send_and_recv(sock, server_addr, bytes(output),
                             timeout_s=args.timeout, retries=args.retries, semantics=args.semantics)

        pos = 2
        resp = BookingResponse.deserialize(data, pos)
        print(f"Booking Response: confirmation_id={resp.confirmation_id}, message={resp.message}")

    elif args.command == "update":
        upd = Update(confirmation_id=args.confirmation_id, offset=args.offset)

        rid = next_req_id()
        output = bytearray([RequestType.UPDATE.value, rid])
        output.extend(upd.serialize())

        data = send_and_recv(sock, server_addr, bytes(output),
                             timeout_s=args.timeout, retries=args.retries, semantics=args.semantics)

        pos = 2
        resp = UpdateResponse.deserialize(data, pos)
        ok = "success" if resp.status == 0 else "failure"
        print(f"Update Response: {ok}, message={resp.message}")

    elif args.command == "monitor":
        # 注册监控：发送 [MONITOR][req_id] + duration
        rid = next_req_id()
        mon = Monitor(duration=args.duration)
        output = bytearray([RequestType.MONITOR.value, rid])
        output.extend(mon.serialize())
        sock.sendto(bytes(output), server_addr)

        sock.settimeout(10.0)
        start_time = time.time()
        print(f"Monitoring for {args.duration} seconds...")

        while time.time() - start_time < args.duration:
            try:
                data, _ = sock.recvfrom(4096)
                pos = 2  # 跳过 [resp_type][resp_id]
                facility = FacilityRecord.deserialize(data, pos)
                print(str(facility))
            except socket.timeout:
                continue
            except Exception as e:
                print(f"Error: {e}")

        # 结束时尝试取消订阅（若服务端支持：duration=0）
        try:
            rid = next_req_id()
            cancel = Monitor(duration=0)
            output = bytearray([RequestType.MONITOR.value, rid])
            output.extend(cancel.serialize())
            sock.sendto(bytes(output), server_addr)
        except Exception:
            pass

        print(f"Monitoring ended after {args.duration} seconds.")

    else:
        raise RuntimeError(f"Unknown command: {args.command}")

if __name__ == "__main__":
    main()
