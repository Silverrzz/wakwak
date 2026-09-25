import { accessSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { createServer } from "node:http";

const root = new URL("../target/browser/", import.meta.url);
const types = { "wakwak.js": "text/javascript", "wakwak.worker.js": "text/javascript", "wakwak.wasm": "application/wasm" };
for (const name of Object.keys(types)) accessSync(new URL(name, root));
const index = '<!doctype html><title>WakWak WASM</title><ul>' +
    Object.keys(types).map(name => `<li><a href="${name}">${name}</a></li>`).join("") + '</ul>';

createServer(async (req, res) => {
    res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
    res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
    res.setHeader("Cache-Control", "no-store");
    if (req.method !== "GET" && req.method !== "HEAD") {
        res.writeHead(405, { Allow: "GET, HEAD" }).end();
        return;
    }
    const name = req.url.split("?")[0].slice(1);
    try {
        if (name === "" || name === "index.html") {
            res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" }).end(index);
        } else if (Object.hasOwn(types, name)) {
            const body = await readFile(new URL(name, root));
            res.writeHead(200, { "Content-Type": types[name] }).end(body);
        } else res.writeHead(404).end();
    } catch (error) {
        res.writeHead(error.code === "ENOENT" ? 404 : 500).end();
    }
}).listen(8000, "127.0.0.1", () => console.log("WakWak WASM: http://localhost:8000/ (Ctrl+C to stop)"));
