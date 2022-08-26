import init, { wasm_main } from "./fractal.js";

function touchHandler(event) {
  // Turn touch events into mouse events
  let touches = event.changedTouches,
    first = touches[0],
    type = "";
  switch (event.type) {
    case "touchstart": type = "mousedown"; break;
    case "touchmove": type = "mousemove"; break;
    case "touchend": type = "mouseup"; break;
    default: return;
  }

  let simulatedEvent = document.createEvent("MouseEvent");
  simulatedEvent.initMouseEvent(type, true, true, window, 1,
    first.screenX, first.screenY,
    first.clientX, first.clientY, false,
    false, false, false, 0/*left*/, null);

  first.target.dispatchEvent(simulatedEvent);
  event.preventDefault();
}

async function run() {
  document.addEventListener("touchstart", touchHandler, true);
  document.addEventListener("touchmove", touchHandler, true);
  document.addEventListener("touchend", touchHandler, true);
  document.addEventListener("touchcancel", touchHandler, true);
  await init();
  wasm_main();
}

run();