import argparse
import socket
import ssl
import time


def receive_exact(sock: ssl.SSLSocket, length: int) -> bytes:
    output = bytearray()
    while len(output) < length:
        chunk = sock.recv(length - len(output))
        if not chunk:
            raise EOFError
        output.extend(chunk)
    return bytes(output)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=19878)
    parser.add_argument("--seconds", type=float, default=2.5)
    args = parser.parse_args()

    context = ssl.create_default_context()
    context.check_hostname = False
    context.verify_mode = ssl.CERT_NONE
    context.set_alpn_protocols(["h2"])

    for _ in range(100):
        try:
            tcp = socket.create_connection(("127.0.0.1", args.port), timeout=1)
            break
        except OSError:
            time.sleep(0.05)
    else:
        raise RuntimeError("Nginx did not become ready")

    with context.wrap_socket(tcp, server_hostname="localhost") as sock:
        if sock.selected_alpn_protocol() != "h2":
            raise RuntimeError("Nginx did not negotiate h2")
        sock.sendall(
            b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"
            b"\x00\x00\x00\x04\x00\x00\x00\x00\x00"
        )

        deadline = time.monotonic() + args.seconds
        frames: list[tuple[int, int, int, int]] = []
        proactive_pings = 0
        while time.monotonic() < deadline:
            sock.settimeout(max(0.05, deadline - time.monotonic()))
            try:
                header = receive_exact(sock, 9)
                length = int.from_bytes(header[:3], "big")
                payload = receive_exact(sock, length)
            except (socket.timeout, EOFError):
                break
            frame_type = header[3]
            flags = header[4]
            stream_id = int.from_bytes(header[5:9], "big") & 0x7FFF_FFFF
            frames.append((frame_type, flags, stream_id, length))
            if frame_type == 0x4 and flags & 0x1 == 0:
                sock.sendall(b"\x00\x00\x00\x04\x01\x00\x00\x00\x00")
            if frame_type == 0x6 and flags & 0x1 == 0:
                proactive_pings += 1
                sock.sendall(
                    b"\x00\x00\x08\x06\x01\x00\x00\x00\x00" + payload
                )

    print(
        f"idle_window_ms={int(args.seconds * 1000)}; "
        f"proactive_pings={proactive_pings}; frames={frames}"
    )


if __name__ == "__main__":
    main()
