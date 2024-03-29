var socket = null;

var last_message;

function post_message(message) {
    last_message = message;
    socket.send(JSON.stringify(message));
}

function create_socket() {
    socket = new WebSocket("ws://127.0.0.1:9001");

    socket.onopen = function (e) {
        var temp = document.getElementById("connection-status-indicator");
        temp.textContent = "Connected";
        temp.style.color = "green";
        // alert("[open] Connection established");

        var elems = document.getElementsByClassName("require-connection");
        for(var i = 0;i < elems.length;i ++){
            elems[i].disabled = false;
        }
    };
    
    socket.onmessage = function (event) {
        var obj = JSON.parse(event.data);
        if(Object.hasOwn(obj, 'Error')) { 
            alert(`[message] Error received from server: ${event.data}`);
        } else if(Object.hasOwn(obj, 'LoadFileResult')) {
            load_save_state(obj.LoadFileResult.data);
        } else if(Object.hasOwn(obj, 'RunSCResult')) {
            handle_sc_response(obj.RunSCResult.to_convert);
        } else if(Object.hasOwn(obj, 'CompilationResult')) {
            handle_comp_response(obj.CompilationResult.result);
        } else if(obj == "RequestOverwrite") {
            if(confirm("Overwrite file?")) {
                post_message({SaveFile: {file_path: document.getElementById("save-file-location").value, data: get_state_for_save(), overwrite: true}});
            }
        } else if (obj == "Success") {
            if(Object.hasOwn(last_message, 'LoadProgram')) { 
                handle_program_area_compile_success();
            }
        } else if(Object.hasOwn(obj, 'TableResult')) {
            if(obj.TableResult.table === null) {
                // TODO: Proper error handling
                alert("compilation error")
            } else {
                load_table_table(obj.TableResult.table);
            }
        } else {
            alert(`[message] Unknown data received from server: ${event.data}`);
        }
    };
    
    socket.onclose = function (event) {
        var temp = document.getElementById("connection-status-indicator");
        temp.textContent = "Not Connected";
        temp.style.color = "red";
        if (event.wasClean) {
            // alert(`[close] Connection closed cleanly, code=${event.code} reason=${event.reason}`);
        } else {
            // e.g. server process killed or network down
            // event.code is usually 1006 in this case
            // alert('[close] Connection died');
        }
        var elems = document.getElementsByClassName("require-connection");
        for(var i = 0;i < elems.length;i ++){
            elems[i].disabled = true;
        }
        socket = null;
    };
}

function get_state_for_save() {
    save_spreadsheet_state();
    save_table_state();
    handle_program_manager_save();

    var obj = {};
    obj.bottom_bar_state = bottom_bar_state;
    obj.spreadsheet_states = spreadsheet_states;
    obj.table_sources = table_sources;
    obj.current_index = current_spreadsheet_id;
    obj.programs = programs;
    return JSON.stringify(obj);
}

function load_save_state(data) {
    var startTime = performance.now();

    var obj = JSON.parse(data);
    bottom_bar_state = obj.bottom_bar_state;
    spreadsheet_states = obj.spreadsheet_states;
    table_sources = obj.table_sources;

    current_spreadsheet_id = obj.current_index;
    current_table_id = +document.getElementById("lexicon-table-select").value;

    current_spreadsheet_state = spreadsheet_states[current_spreadsheet_id];

    programs = obj.programs;

    //TODO: Organize this better?
    clear_program_dropdown();
    insert_program_dropdown(obj.programs); 

    delete_table_display();
    delete_spreadsheet();
    delete_bottom_bar();
    
    compile_programs();

    compile_tables();

    create_spreadsheet();
    create_bottom_bar();

    handle_bottom_bar_button(obj.current_index);

    var endTime = performance.now();
    console.log(`Call to load_save_state took ${endTime - startTime} milliseconds`);
}

function send_compile_request(name, program) {
    post_message({ LoadProgram: { name: name, contents: program }});
}

function compile_programs() {
    var keys = Object.keys(programs);
    for (var i = 0; i < keys.length; i++) {
        send_compile_request(keys[i], programs[keys[i]][0]);
    }
}

function send_sc_request(array, program) {
    post_message({ RunSC: { program_name: program, to_convert: array }});
}

function try_compile(program) {
    try {
        post_message({ TryCompile: { program: program }});
        return true;
    } catch(err) {
        return false;
    }
}

document.getElementById("button-connect").addEventListener("mousedown", function(e) { if(socket == null) create_socket(); });

var elems = document.getElementsByClassName("require-connection");
for(var i = 0;i < elems.length;i ++){
    elems[i].disabled = true;
}