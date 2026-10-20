import init, { setup_shared, connect_tab } from "../.wasm_pack/client.js";

let wasm_initialized = false;

onconnect = async function (event) {
  // if two tabs connect at once, make second wait for in-progress init
  await navigator.locks.request("sync_init", async (_lock) => {
    if (!wasm_initialized) {
      await init();
      // await setup_shared("ws://localhost:3036/ws");
      await setup_shared();
      wasm_initialized = true;
    }
  });
  connect_tab(event.ports[0]);
  console.log("browser tab connected to shared worker");
}
