import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { fileURLToPath } from "node:url";

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
