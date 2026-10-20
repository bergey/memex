import init, { observability, worker } from "../.wasm_pack/client.js";

let wasm_initialized = false;

onconnect = async function (event) {
  if (!wasm_initialized) {
    // TODO await lock if init is in flight
    await init();
    observability();
    wasm_initialized = true;
  }
  const broadcast = new BroadcastChannel('shared_worker');
  event.ports[0].onmessage = function (e) {
    const reply = worker(e.data);
    console.log({recv: e.data, reply});
    broadcast.postMessage(reply);
  }
}
