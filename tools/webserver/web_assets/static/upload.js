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
    websocket.binaryType = 'arraybuffer';
    websocket.onopen = websocket_on_open;
    websocket.onclose = websocket_on_close;
    websocket.onmessage = websocket_on_message;
    websocket.onerror = websocket_on_error;
}

function websocket_on_open(event) {
    console.log("[websocket] open");
    console.log(event);

    send_head_packet();
}

function websocket_on_close(event) {
    console.log("[websocket] close");
    console.log(event);

    if (data === null) {
        upload_notify({
            "done": true,
            "progress": 1.0,
            "error": "",
        });
    } else {
        upload_notify({
            "done": true,
            "progress": 1.0,
            "error": "The connection to the server closed unexpectedly",
        });
        clean_up();
    }
}

function websocket_on_message(event) {
    console.log("[websocket] message");
    console.log(event);

    if (event.data.byteLength !== 1) {
        upload_notify({
            "done": true,
            "progress": bytes_so_far / data.length,
            "error": "Recieved an unknown response.",
        });

        clean_up();
        return;
    }

    let view = new DataView(event.data);
    let response = view.getUint8(0);

    const NEXT = 0x11;
    const FAIL = 0x22;
    const DONE = 0x33;

    if (response === DONE) {
        upload_notify({
            "done": false,
            "progress": 1.0,
            "error": "",
        });
        clean_up();
    } else if (response === NEXT) {
        upload_notify({
            "done": false,
            "progress": bytes_so_far / data.length,
            "error": "",
        });

        send_next_packet();
    } else {
        let message = "The server reported an internal error.";
        if (response !== FAIL) message = "Recieved an unknown response.";

        upload_notify({
            "done": true,
            "progress": bytes_so_far / data.length,
            "error": message,
        });

        clean_up();
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

function int_to_array(number) {
    let a = (number >> 24) & 0xFF;
    let b = (number >> 16) & 0xFF;
    let c = (number >> 8) & 0xFF;
    let d = number & 0xFF;
    return new Uint8Array([ a, b, c, d ]);
}

function send_head_packet() {
    let packet_count = Math.ceil(data.length / packet_size)
    let message = int_to_array(packet_count);
    websocket.send(message);
}

function send_next_packet() {
    let start = bytes_so_far;
    let end = Math.min(start + packet_size, data.length);

    let length = end - start;
    if (length === 0) return;

    bytes_so_far += length;
    websocket.send(data.subarray(start, end));
}
