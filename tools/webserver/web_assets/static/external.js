let socket = null;

function try_open_websocket() {
    if (socket !== null) return false;

    try {
        socket = new WebSocket("ws://" + window.location.host + "/serial");
        socket.onopen = () => report_websocket_state("open")
        socket.onclose = () => report_websocket_state("closed")
        socket.onerror = () => report_websocket_state("closed")
        socket.onmessage = (args) => recieve_websocket_data(args.data + "")
    } catch {
        report_websocket_state("closed")
    }
}

function send_websocket_data(text) {
    if (socket === null) {
        report_websocket_state("closed");
        return;
    }

    socket.send(text);
}

function send_metrics_request() {
    if (socket === null) {
        report_websocket_state("closed");
        return;
    }

    socket.send("metrics\n");
}
