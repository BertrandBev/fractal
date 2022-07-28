import init, { wasm_main } from "./fractal.js";

async function run() {
  await init();
  wasm_main();
}

run();