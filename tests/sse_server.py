#!/usr/bin/env python3
"""
Simple SSE (Server-Sent Events) test server that sends periodic events.
Used for testing the wasmCloud SSE provider.
"""

import http.server
import json
import sys
import threading
import time
from datetime import datetime


class SSEHandler(http.server.BaseHTTPRequestHandler):
    """HTTP handler that serves Server-Sent Events."""

    def do_GET(self):
        if self.path == "/events":
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Cache-Control", "no-cache")
            self.send_header("Connection", "keep-alive")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()

            client_id = id(self)
            print(f"Client {client_id} connected from {self.client_address}")

            message_count = 0
            try:
                while True:
                    message_count += 1

                    # Send SSE event
                    event_data = json.dumps({
                        "type": "test",
                        "count": message_count,
                        "timestamp": datetime.utcnow().isoformat(),
                        "message": f"Test message #{message_count}"
                    })

                    sse_event = f"data: {event_data}\n\n"
                    self.wfile.write(sse_event.encode("utf-8"))
                    self.wfile.flush()
                    print(f"Sent to client {client_id}: {event_data}")

                    time.sleep(3)

            except (BrokenPipeError, ConnectionResetError):
                print(f"Client {client_id} disconnected")
            except Exception as e:
                print(f"Error with client {client_id}: {e}")
        elif self.path == "/health":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(b'{"status":"ok"}')
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        """Override to use print instead of stderr."""
        print(f"[HTTP] {args[0]}")


def main():
    host = "127.0.0.1"
    port = 8765

    server = http.server.HTTPServer((host, port), SSEHandler)
    print(f"Starting SSE test server on http://{host}:{port}/events")
    print("Press Ctrl+C to stop")
    print("-" * 50)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nServer stopped")
        server.server_close()


if __name__ == "__main__":
    main()
