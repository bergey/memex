import init, { start } from "../.wasm_pack/client.js";

(async function () {
  await init();
  start("ws://localhost:3036/ws");
})()

async function registerServiceWorker() {
  if ("serviceWorker" in navigator) {
    try {
      const registration = await navigator.serviceWorker.register(
        new URL("./service-worker.ts", import.meta.url),
        {
          scope: "/",
          type: "module",
        }
      );
      if (process.env.NODE_ENV !== "production") {
        if (registration.installing) {
          console.log("Service worker installing");
        } else if (registration.waiting) {
          console.log("Service worker installed");
        } else if (registration.active) {
          console.log("Service worker active");
        }
      }
    } catch (error) {
      console.error(`Registration failed with ${error}`);
    }
  }
}
registerServiceWorker();
