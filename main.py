#!/usr/bin/env python3
"""
Solana Titan Engine — Server & API Bridge
Runs on Port 8930
"""

import os
import sys
import json
import urllib.parse
from http.server import HTTPServer, SimpleHTTPRequestHandler
from client.simple_dual_strike import SIMPLE_ENGINE
from client.sophisticated_swarm import TITAN_SWARM_ENGINE

BASE_DIR = os.path.dirname(os.path.abspath(__file__))
UI_DIR = os.path.join(BASE_DIR, "ui")

class TitanHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=UI_DIR, **kwargs)

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        if path == "/api/dual_strike/status":
            self.send_json(SIMPLE_ENGINE.active_position or {"status": "IDLE"})
            return

        elif path == "/api/swarm/status":
            self.send_json(TITAN_SWARM_ENGINE.get_swarm_status())
            return

        elif path == "/api/rust_code":
            query = urllib.parse.parse_qs(parsed.query)
            target = query.get("target", ["dual_strike"])[0]
            if target == "titan_swarm":
                file_path = os.path.join(BASE_DIR, "programs", "titan_swarm", "src", "lib.rs")
            else:
                file_path = os.path.join(BASE_DIR, "programs", "dual_strike", "src", "lib.rs")
            
            try:
                with open(file_path, "r") as f:
                    content = f.read()
                self.send_response(200)
                self.send_header("Content-Type", "text/plain")
                self.send_header("Content-Length", str(len(content)))
                self.end_headers()
                self.wfile.write(content.encode("utf-8"))
            except Exception as e:
                self.send_json({"error": str(e)}, 500)
            return

        elif path == "/":
            self.path = "/index.html"
            return super().do_GET()
        else:
            return super().do_GET()

    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        if path == "/api/dual_strike/launch":
            res = SIMPLE_ENGINE.execute_dual_strike_launch()
            # remove non-serializable curve object
            safe_res = dict(res)
            safe_res.pop("curve", None)
            self.send_json(safe_res)
            return

        elif path == "/api/dual_strike/step":
            content_len = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_len).decode("utf-8") if content_len > 0 else "{}"
            data = json.loads(body) if body else {}
            elapsed = float(data.get("elapsed", 1.0))
            sim = SIMPLE_ENGINE.simulate_public_inflow(elapsed)
            self.send_json(sim or {"status": "NO_POSITION"})
            return

        elif path == "/api/dual_strike/exit":
            res = SIMPLE_ENGINE.force_blitz_exit()
            self.send_json(res)
            return

        elif path == "/api/swarm/launch":
            res = TITAN_SWARM_ENGINE.launch_swarm_campaign()
            self.send_json(res)
            return

        elif path == "/api/swarm/step":
            content_len = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_len).decode("utf-8") if content_len > 0 else "{}"
            data = json.loads(body) if body else {}
            elapsed = float(data.get("elapsed", 1.0))
            st = TITAN_SWARM_ENGINE.simulate_swarm_step(elapsed)
            self.send_json(st)
            return

        else:
            self.send_error(404, "Unknown API Route")

    def send_json(self, data, status=200):
        body = json.dumps(data).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

def main():
    port = 8930
    if len(sys.argv) > 2 and sys.argv[1] == "--port":
        port = int(sys.argv[2])

    server_address = ("0.0.0.0", port)
    httpd = HTTPServer(server_address, TitanHandler)
    print("=" * 70)
    print("  👑 SOLANA TITAN ENGINE — DUAL-STRIKE & SWARM ORCHESTRATOR")
    print("=" * 70)
    print(f"[*] Port: {port}")
    print(f"[*] Local UI: http://localhost:{port}")
    print(f"[*] Rust Programs: dual_strike & titan_swarm (Zero-Copy CPI)")
    print(f"[*] Reverse-Engineered Bytecode: artifacts/program_2zwmf.so (59KB)")
    print("=" * 70)

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\n[!] Shutting down Titan Engine...")
        httpd.server_close()
        sys.exit(0)

if __name__ == "__main__":
    main()
