#!/usr/bin/env python3
"""
Deskemoji Review Local Server
启动后提供静态文件服务 + /api/save /api/load 接口，
让交互式 HTML 决策板能自动持久化数据到本地 JSON 文件。
"""

import json
import os
from http.server import HTTPServer, SimpleHTTPRequestHandler
from urllib.parse import urlparse

# 配置
PORT = 8765
ROOT_DIR = os.path.dirname(os.path.abspath(__file__))
DATA_DIR = os.path.join(ROOT_DIR, ".review-data")
os.makedirs(DATA_DIR, exist_ok=True)

class ReviewHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        super().end_headers()

    def do_OPTIONS(self):
        self.send_response(204)
        self.end_headers()

    def do_GET(self):
        parsed = urlparse(self.path)
        if parsed.path.startswith("/api/load"):
            filename = os.path.basename(parsed.query) or "default"
            filepath = os.path.join(DATA_DIR, f"{filename}.json")
            if os.path.exists(filepath):
                with open(filepath, "r", encoding="utf-8") as f:
                    data = f.read()
            else:
                data = "{}"
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(data.encode("utf-8"))
            return
        super().do_GET()

    def do_POST(self):
        parsed = urlparse(self.path)
        if parsed.path == "/api/save":
            content_length = int(self.headers.get("Content-Length", 0))
            body = self.rfile.read(content_length).decode("utf-8")
            payload = json.loads(body)
            filename = payload.get("filename", "default")
            filepath = os.path.join(DATA_DIR, f"{filename}.json")
            with open(filepath, "w", encoding="utf-8") as f:
                json.dump(payload.get("data", {}), f, ensure_ascii=False, indent=2)
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"ok": True, "savedTo": filepath}).encode("utf-8"))
            print(f"[SAVED] {filepath}")
            return
        self.send_response(404)
        self.end_headers()

if __name__ == "__main__":
    os.chdir(ROOT_DIR)
    server = HTTPServer(("127.0.0.1", PORT), ReviewHandler)
    print(f"=" * 50)
    print(f"Deskemoji Review Server 已启动")
    print(f"访问地址: http://127.0.0.1:{PORT}")
    print(f"数据保存目录: {DATA_DIR}")
    print(f"=" * 50)
    print("按 Ctrl+C 停止服务")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n服务器已停止")
