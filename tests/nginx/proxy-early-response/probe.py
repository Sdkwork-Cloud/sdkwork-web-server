import argparse
import socket
import threading
import time


def receive_headers(sock: socket.socket) -> bytes:
    output = bytearray()
    while not output.endswith(b"\r\n\r\n"):
        chunk = sock.recv(1)
        if not chunk:
            raise EOFError("connection ended before headers")
        output.extend(chunk)
        if len(output) > 8192:
            raise RuntimeError("header block exceeded probe limit")
    return bytes(output)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--proxy-port", type=int, default=19879)
    parser.add_argument("--upstream-port", type=int, default=19880)
    args = parser.parse_args()

    ready = threading.Event()
    result: dict[str, object] = {}

    def serve_upstream() -> None:
        with socket.socket() as listener:
            listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            listener.bind(("127.0.0.1", args.upstream_port))
            listener.listen(1)
            ready.set()
            connection, _ = listener.accept()
            with connection:
                connection.settimeout(2)
                headers = receive_headers(connection)
                result["upstream_headers"] = headers.decode("ascii", "replace")
                connection.sendall(
                    b"HTTP/1.1 401 Early\r\n"
                    b"Content-Length: 9\r\n"
                    b"Connection: keep-alive\r\n\r\n"
                    b"early-401"
                )
                trailing = 0
                upstream_eof = False
                deadline = time.monotonic() + 2
                while time.monotonic() < deadline:
                    try:
                        chunk = connection.recv(4096)
                    except socket.timeout:
                        break
                    if not chunk:
                        upstream_eof = True
                        break
                    trailing += len(chunk)
                result["upstream_trailing_bytes"] = trailing
                result["upstream_eof"] = upstream_eof

    upstream = threading.Thread(target=serve_upstream, daemon=True)
    upstream.start()
    if not ready.wait(2):
        raise RuntimeError("upstream did not bind")

    proxy_deadline = time.monotonic() + 5
    while time.monotonic() < proxy_deadline:
        try:
            client = socket.create_connection(("127.0.0.1", args.proxy_port), timeout=0.25)
            break
        except OSError:
            time.sleep(0.05)
    else:
        raise RuntimeError("Nginx did not become ready")

    with client:
        client.settimeout(2)
        client.sendall(
            b"POST /early HTTP/1.1\r\n"
            b"Host: localhost\r\n"
            b"Content-Length: 100000\r\n\r\n"
            b"seed"
        )
        response = bytearray()
        client_eof = False
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            try:
                chunk = client.recv(4096)
            except socket.timeout:
                break
            if not chunk:
                client_eof = True
                break
            response.extend(chunk)
            if len(response) > 1024 * 1024:
                raise RuntimeError("response exceeded probe limit")

    upstream.join(timeout=3)
    response_text = response.decode("ascii", "replace")
    status = response_text.split("\r\n", 1)[0] if response else "none"
    connection_close = any(
        line.lower() == "connection: close" for line in response_text.split("\r\n")
    )
    body_complete = response.endswith(b"early-401")
    print(
        f"status={status!r}; body_complete={body_complete}; "
        f"connection_close={connection_close}; client_eof={client_eof}; "
        f"upstream_eof={result.get('upstream_eof')}; "
        f"upstream_trailing_bytes={result.get('upstream_trailing_bytes')}"
    )


if __name__ == "__main__":
    main()
