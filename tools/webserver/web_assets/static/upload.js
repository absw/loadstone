var data = null;
var bytes_so_far = 0;
var websocket = null;
const packet_size = 128;

function base64_to_array(base64) {
    var string = atob(base64);
    var bytes = new Uint8Array(string.length);
    for (var i = 0; i < string.length; i++) {
        bytes[i] = string.charCodeAt(i);
    }
    return bytes;
}

function upload_start(base64) {
    if (data !== null) return;
    data = base64_to_array(base64);
    bytes_so_far = 0;

    websocket = new WebSocket("ws://" + window.location.host + "/upload");
    websocket.onopen = websocket_on_open;
    websocket.onclose = websocket_on_close;
    websocket.onmessage = websocket_on_message;
    websocket.onerror = websocket_on_error;
}

function websocket_on_open(event) {
    console.log("[websocket] open");
    console.log(event);

    send_next_packet();
}

function websocket_on_close(event) {
    console.log("[websocket] close");
    console.log(event);

    upload_notify({
        "done": true,
        "progress": bytes_so_far / data.length,
        "error": "websocket closed unexpectedly D:",
    });

    clean_up();
}

function websocket_on_message(event) {
    console.log("[websocket] message");
    console.log(event);

    upload_notify({
        "done": false,
        "progress": bytes_so_far / data.length,
        "error": "",
    });

    if (bytes_so_far == data.length) {
        websocket.close();
    } else {
        send_next_packet();
    }
}

function websocket_on_error(event) {
    console.log("[websocket] error");
    console.log(event);

    upload_notify({
        "done": true,
        "progress": 0,
        "error": "websocket bad D:",
    });

    websocket = null;
}

function clean_up() {
    websocket = null;
    data = null;
    bytes_so_far = 0;
}

function send_next_packet() {
    let start = bytes_so_far;
    let end = Math.min(start + packet_size, data.length - 1);

    let length = end - start;
    if (length === 0) return;

    bytes_so_far += length;
    websocket.send(data.subarray(start, end));
}
