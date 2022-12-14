var programs = {};

function update_textarea(comp) {
    var area = document.getElementById("program-textarea");
    var element = document.getElementById("textarea-renderer-container");
    var numbers = document.getElementById("line-numbers");
    var lines = area.value.split("\n");
    element.replaceChildren();
    numbers.replaceChildren();
    syntax_highlighter(element, area.value, []);

    for(var i = 0;i < lines.length;i ++) {
        var number = document.createElement("p");
        number.textContent = i + 1 + "";
        number.className = "line-number unselectable";
        numbers.appendChild(number);
    }

    scroll_textarea();

    if(comp) {
        current_try_compilation_contents = area.value;
        current_try_compilation_contents_counter = current_try_compilation_counter + 1;
        try_try_compile();
    }
}

var current_try_compilation_contents = "";
var current_try_compilation_contents_counter = 0;
var current_try_compilation_counter = 0;
var current_try_compilation_response_counter = 0;
var current_error_line = -1;

function try_try_compile() {
    if(current_try_compilation_counter === current_try_compilation_response_counter && current_try_compilation_contents_counter > current_try_compilation_counter) {
        if(try_compile(current_try_compilation_contents))
            current_try_compilation_counter++;
    }
}

function handle_comp_response(response) {
    var element = document.getElementById("compilation-status");
    var prev_error_line = current_error_line;
    if(response === null) {
        element.style.color = "green";
        element.textContent = "Compilation Status: Compilation Success";
        current_error_line = -1;
    } else {
        element.style.color = "red";
        var str = "Compilation Status: ";
        str += response.error_type + " - " + response.error_message + ", line #" + response.line_number_user_program.Raw;
        current_error_line = response.line_number_user_program.Raw - 1;
        
        element.textContent = str;
    }
    if(prev_error_line != current_error_line) update_textarea(false);
    current_try_compilation_response_counter++;
    try_try_compile();
}

function scroll_textarea() {
    var val = document.getElementById("program-textarea").scrollTop;
    var val2 = document.getElementById("program-textarea").scrollLeft;
    var element = document.getElementById("textarea-renderer-container");
    element.style.top = (1.5-val) + "px";
    element.style.right = (val2-30) + "px";

    element = document.getElementById("line-numbers");
    element.style.top = (1.5-val) + "px";
}

function populate_program_dropdown() {
    var element = document.getElementById("programs-selector");
    element.replaceChildren();

    var temp = Object.getOwnPropertyNames(programs);
    for(var i = 0;i < temp.length;i ++){
        var temp_element = document.createElement("option");
        temp_element.textContent = temp[i];
        element.appendChild(temp_element);
    }
}

function clear_program_dropdown() {
    var element = document.getElementById("programs-selector");
    element.replaceChildren();
}

function insert_program_dropdown(obj) {
    var keys = Object.keys(obj);

    for (var i = 0; i < keys.length; i++) {
        add_program_to_dropdown(keys[i]);
        //TODO: Save selected program?
        if(i == keys.length - 1) {
            handle_program_manager_selection_change(); 
        }
    }
}

function add_program_to_dropdown(name) {
    var element = document.getElementById("programs-selector");
    var to_add = document.createElement("option");
    to_add.id = "program-selector-option-" + name;
    to_add.textContent = name;
    element.append(to_add);
    element.value = name;
}

function get_test_area_content () {
    var temp = document.getElementById("program-manager-test-area").value.split("\n");
    var to_return = [];
    for(var i = 0;i < temp.length;i ++) {
        if(temp[i] != "") {
            to_return.push(temp[i].split("=>")[0].trim());
        }
    }
    return to_return.join("\n");
}

//This marks what the name that is internally used for indentifying the program with the backend
var send_test_words_flag = "";

function handle_program_manager_run() {
    var element = document.getElementById("program-manager-rename");
    var name = element.value;
    var reset_test_area  = get_test_area_content();
    document.getElementById("program-manager-test-area").value = reset_test_area;
    var program_contents = document.getElementById("program-textarea").value;
    if(name != "") {
        name = name.replace(/\s/g, '_');
    }

    var request_prog_name = name == "" ? "test" : name + "_test";
    send_compile_request(request_prog_name, program_contents);

    send_test_words_flag = request_prog_name;
}

function handle_program_manager_save() {
    var element = document.getElementById("program-manager-rename");
    var name = element.value;
    var reset_test_area  = get_test_area_content();
    document.getElementById("program-manager-test-area").value = reset_test_area;
    var program_contents = document.getElementById("program-textarea").value;
    if(name != "") {
        name = name.replace(/\s/g, '_');
        if(programs[name] === undefined) {
            add_program_to_dropdown(name);
        }
        programs[name] = [program_contents, reset_test_area];
    }

    send_compile_request(name == "" ? "test" : name, program_contents);

    mark_dirty();

    send_test_words_flag = name == "" ? "test" : name;
}

function handle_program_area_compile_success() {
    if(send_test_words_flag != "") {
        var temp = document.getElementById("program-manager-test-area").value.split("\n");
        for(var i = 0;i < temp.length;i ++) {
            if(temp[i] != "") {
                add_program_test_area_conversion(send_test_words_flag, temp[i], i);
            }
        }
    }
}

function handle_program_manager_selection_change() {
    //TODO: Check if prev is unsaved

    var element = document.getElementById("programs-selector");
    document.getElementById("program-manager-rename").value = element.value;

    var to_load = programs[element.value];
    document.getElementById("program-textarea").value = to_load[0];
    document.getElementById("program-manager-test-area").value = to_load[1];

    update_textarea(true);
}

populate_program_dropdown();

document.getElementById("program-textarea").addEventListener("input", () => update_textarea(true));
document.getElementById("program-textarea").addEventListener("scroll", scroll_textarea);

//This handles (poorly) helpful features of the IDE like auto indentation
//TODO: Refactor and rewrite all of this
document.getElementById('program-textarea').addEventListener('keydown', function(e) {
    if (e.key == 'Tab') {//based on https://stackoverflow.com/questions/6637341/use-tab-to-indent-in-textarea
        e.preventDefault();
        var start = this.selectionStart;
        var end = this.selectionEnd;
  
        this.value = this.value.substring(0, start) +
          "    " + this.value.substring(end);
  
        this.selectionStart =
          this.selectionEnd = start + 4;

        update_textarea(true);
    }
    if (e.key == 'Enter') {
        e.preventDefault();
        var start = this.selectionStart;
        var end = this.selectionEnd;

        var i = this.selectionStart;
        for(; i > 0; i --) {
            if(this.value[i] == "\n") break;
        }
        if(i != 0) i++;
        for(var c = 0; i < this.value.length;i ++, c++) {
            if(this.value[i] != " ") break;
        }

        var prev_start = -1;
        var prev_end = -1;
        var q = 0;

        var temp = this.value.substring(0, start).split("\n");
        temp = temp[temp.length-1];
        var trimmed = temp.trim();
        if(trimmed == "feature_def") c += 4;
        if(trimmed == "symbols") c += 4;
        if(trimmed == "diacritics") c += 4;
        if(trimmed == "rules") c += 4;
        if(trimmed == "rule") c += 4;
        if(trimmed == "end") {
            q = (c >= 4 ? 4 : c);

            prev_end = start - 3;
            prev_start = start - 3 - q;

            c -= 4;
            if(c < 0) c = 0;
            flag = true;
        }
  
        if(prev_start != -1 && prev_end != -1) this.value = this.value.substring(0, prev_start) + this.value.substring(prev_end);

        this.value = this.value.substring(0, start - q) +
          "\n" + " ".repeat(c) + this.value.substring(end - q);
  
        this.selectionStart =
          this.selectionEnd = start + 1 + c - q;

        update_textarea(true);
    }
});

update_textarea(true);

document.getElementById("program-manager-button-exit").addEventListener("mousedown", function(e) { 
    var elem = document.getElementById("program-menu");
    if(elem.style.display == "") {
        elem.style.display = "none";
    } else {
        elem.style.display = "";
    }
});

document.getElementById("program-manager-button-save").addEventListener("mousedown", () => handle_program_manager_save());
document.getElementById("program-manager-button-comprun").addEventListener("mousedown", () => handle_program_manager_run());
document.getElementById("programs-selector").addEventListener("input", () => handle_program_manager_selection_change());