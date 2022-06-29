import init, { wasm_main } from "./pkg/cornelis.js";

async function run() {
  await init();
  wasm_main();
}

run();
