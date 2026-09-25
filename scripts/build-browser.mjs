import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readdirSync } from "node:fs";
import { delimiter, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const sdk = resolve(process.env.EMSDK || fileURLToPath(new URL("../target/emsdk", import.meta.url)));
if (existsSync(join(sdk, ".emscripten"))) {
    process.env.EMSDK = sdk;
    process.env.EM_CONFIG ??= join(sdk, ".emscripten");
    process.env.PATH = [join(sdk, "upstream/emscripten"), join(sdk, "upstream/bin"), process.env.PATH || ""].join(delimiter);
    const pythonRoot = join(sdk, "python");
    if (process.platform === "win32" && !process.env.EMSDK_PYTHON && existsSync(pythonRoot)) {
        const python = readdirSync(pythonRoot).map(name => join(pythonRoot, name, "python.exe"))
            .find(path => existsSync(path));
        if (python) process.env.EMSDK_PYTHON = python;
    }
    console.log(`Emscripten SDK: ${sdk}`);
}
process.chdir(fileURLToPath(new URL("..", import.meta.url)));
const target = "target/browser-build";
const output = "target/browser";
const result = spawnSync("cargo", [
    `+${process.env.WAKWAK_TOOLCHAIN || "nightly"}`,
    "build", "--release", "--locked",
    "--target", "wasm32-unknown-emscripten",
    "--target-dir", target,
    "-Zbuild-std=std,panic_abort",
], { stdio: "inherit" });

if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);

mkdirSync(output, { recursive: true });
for (const name of ["wakwak.js", "wakwak.wasm"]) {
    copyFileSync(`${target}/wasm32-unknown-emscripten/release/${name}`, `${output}/${name}`);
}
copyFileSync("browser/wakwak.worker.js", `${output}/wakwak.worker.js`);
copyFileSync("LICENSE", `${output}/LICENSE`);
console.log(`Browser build: ${output}`);
