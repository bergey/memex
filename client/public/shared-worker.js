onconnect = function (event) {
  const broadcast = new BroadcastChannel('shared_worker');
  event.ports[0].onmessage = function (e) {
    const reply = e.data + 1;
    console.log({recv: e.data, reply});
    broadcast.postMessage(reply);
  }
}
