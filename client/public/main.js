import init, { start } from "../.wasm_pack/client.js";

(async function () {
  await init();
  start("ws://localhost:3003/ws");
})()

