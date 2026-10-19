onconnect = function (event) {
  const broadcast = new BroadcastChannel('shared_worker');
  event.ports[0].onmessage = function (e) {
    console.log(e.data);
    broadcast.postMessage(e.data[0] + 1);
  }
}
