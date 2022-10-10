var socket = null;

function post_message(message) {
    socket.send(JSON.stringify(message));
}

function create_socket() {
    socket = new WebSocket("ws://127.0.0.1:9001");

    socket.onopen = function (e) {
        var temp = document.getElementById("connection-status-indicator");
        temp.textContent = "Connected";
        temp.style.color = "green";
        alert("[open] Connection established");

        var elems = document.getElementsByClassName("require-connection");
        for(var i = 0;i < elems.length;i ++){
            elems[i].disabled = false;
        }
    };
    
    socket.onmessage = function (event) {
        var obj = JSON.parse(event.data);
        console.log(event.data);
        console.log(obj);
        if(Object.hasOwn(obj, 'Error')) { 
            alert(`[message] Error received from server: ${event.data}`);
        } else if(obj == "RequestOverwrite") {
            if(confirm("Overwrite file?")) {
                post_message({SaveFile: {file_path: document.getElementById("save-file-location").value, data: get_state_for_save(), overwrite: true}});
            }
        } else if (obj == "Success") {

        } else {
            alert(`[message] Unknown data received from server: ${event.data}`);
        }
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
        var elems = document.getElementsByClassName("require-connection");
        for(var i = 0;i < elems.length;i ++){
            elems[i].disabled = true;
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

document.getElementById("button-connect").addEventListener("mousedown", function(e) { if(socket == null) create_socket(); });

var elems = document.getElementsByClassName("require-connection");
for(var i = 0;i < elems.length;i ++){
    elems[i].disabled = true;
}