import init, { observability, worker } from "../.wasm_pack/client.js";

let wasm_initialized = undefined;

onconnect = async function (event) {
  // if two tabs connect at once, make second wait for in-progress init
  await navigator.locks.request("sync_init", async (_lock) => {
    if (!wasm_initialized) {
      await init();
      await setup_shared("ws://localhost:3036/ws");
      wasm_initialized = true;
    }
  });
  connect_tab(event.ports[0]);
  /* event.ports[0].onmessage = wasm_initialized; */
}
