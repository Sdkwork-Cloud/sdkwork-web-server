import argparse
import socket
import time


def receive_response(sock: socket.socket) -> tuple[bytes, bytes]:
    headers = bytearray()
    while not headers.endswith(b"\r\n\r\n"):
        chunk = sock.recv(1)
        if not chunk:
            raise EOFError("connection closed before response headers")
        headers.extend(chunk)
        if len(headers) > 8192:
            raise RuntimeError("response headers exceeded test bound")

    content_length = 0
    for line in headers.decode("ascii").split("\r\n"):
        name, separator, value = line.partition(":")
        if separator and name.lower() == "content-length":
            content_length = int(value.strip())
    if content_length > 1024:
        raise RuntimeError("response body exceeded test bound")

    body = bytearray()
    while len(body) < content_length:
        chunk = sock.recv(content_length - len(body))
        if not chunk:
            raise EOFError("connection closed before response body")
        body.extend(chunk)
    return bytes(headers), bytes(body)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=19880)
    parser.add_argument("--age-wait-seconds", type=float, default=1.25)
    args = parser.parse_args()

    for _ in range(100):
        try:
            sock = socket.create_connection(("127.0.0.1", args.port), timeout=1)
            break
        except OSError:
            time.sleep(0.05)
    else:
        raise RuntimeError("Nginx did not become ready")

    with sock:
        sock.settimeout(2)
        sock.sendall(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        first_headers, first_body = receive_response(sock)
        time.sleep(args.age_wait_seconds)
        sock.sendall(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        second_headers, second_body = receive_response(sock)
        eof_after_second = sock.recv(1) == b""

    print(
        f"first_status={first_headers.splitlines()[0]!r}; "
        f"first_body={first_body!r}; "
        f"second_status={second_headers.splitlines()[0]!r}; "
        f"second_body={second_body!r}; "
        f"second_connection_close={b'Connection: close' in second_headers}; "
        f"eof_after_second={eof_after_second}"
    )


if __name__ == "__main__":
    main()
