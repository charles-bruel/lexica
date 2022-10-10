var socket = null;

function create_socket() {
    socket = new WebSocket("ws://127.0.0.1:9001");

    socket.onopen = function (e) {
        var temp = document.getElementById("connection-status-indicator");
        temp.textContent = "Connected";
        temp.style.color = "green";
        alert("[open] Connection established");
    };
    
    socket.onmessage = function (event) {
        alert(`[message] Data received from server: ${event.data}`);
    };
    
    socket.onclose = function (event) {
        var temp = document.getElementById("connection-status-indicator");
        temp.textContent = "Not Connected";
        temp.style.color = "red";
        if (event.wasClean) {
            alert(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
        } else {
            // e.g. server process killed or network down
            // event.code is usually 1006 in this case
            alert('[close] Connection died');
        }
        socket = null;
    };
}

function get_state_for_save() {
    var obj = {};
    obj.bottom_bar_state = bottom_bar_state;
    obj.spreadsheet_state = spreadsheet_states;
    return JSON.stringify(obj);
}

function post_message(message) {
    socket.send(JSON.stringify(message));
}

document.getElementById("button-connect").addEventListener("mousedown", function(e) { if(socket == null) create_socket(); });