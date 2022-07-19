import init, { wasm_main } from "./pkg/fractal.js";

async function run() {
  await init();
  wasm_main();
}

run();