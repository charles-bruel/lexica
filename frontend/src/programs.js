var programs = {};

function update_textarea(comp) {
    var area = document.getElementById("program-textarea");
    var element = document.getElementById("textarea-renderer-container");
    var numbers = document.getElementById("line-numbers");
    var temp = area.value.replace("\t", "    ");
    var lines = temp.split("\n");
    element.replaceChildren();
    numbers.replaceChildren();
    var symbols = [];
    for(var i = 0;i < lines.length;i ++) {
        var number = document.createElement("p");
        number.textContent = i + 1 + "";
        number.className = "line-number unselectable";
        numbers.appendChild(number);

        var type_flag = "";
        var whitespcae_flag = 0;
        var class_flag = (i === current_error_line ? "synhi-err" : "");
        var symbol_flag = false;
        var running = "";
        for(var j = 0;j < lines[i].length;j ++) {
            var char = lines[i][j];
            if(whitespcae_flag === -1 && /\s/.test(char)) {
                if(type_flag === "feature") {
                    class_flag += " synhi-feature";
                }
                if(symbol_flag) {
                    symbols.push(running.trim());
                    symbol_flag = false;
                }
                add_textarea_span(element, running, class_flag);
                running = "";
                whitespcae_flag = 0;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }
            if(/\s/.test(char)) {//is whitespace?
                whitespcae_flag = 1;
            } else {
                whitespcae_flag = -1;
            }

            if(char === "#" || char === "(" || char === ")" || char === "[" || char === "]" || char === "{" || char === "}") {
                if(type_flag === "feature") {
                    class_flag += " synhi-feature";
                }
                if(symbol_flag) {
                    symbols.push(running.trim());
                    symbol_flag = false;
                }
                if(symbols.includes(running.trim())) {
                    class_flag += " synhi-symbol";
                }
                add_textarea_span(element, running, class_flag);
                running = "";
                whitespcae_flag = 0;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }
            if(char === "#"){
                add_textarea_span(element, lines[i].substring(j), "synhi-comment");
                break;
            }
            
            running += char;
            var temp = running.trimStart();

            if(temp == "(" || temp == ")" || temp == "[" || temp == "]" || temp == "{" || temp == "}") {
                class_flag += " synhi-paren";
                add_textarea_span(element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }

            var end_flag =  j == lines[i].length - 1 || /\s/.test(lines[i][j + 1]);
            if(!end_flag) continue;

            if(temp == "feature_def" || temp == "symbols" || temp == "rules" || temp == "rule" || temp == "diacritics" || temp == "end") {
                class_flag += " synhi-top-level";
                add_textarea_span(element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }

            if(temp == "feature" || temp == "switch" || temp == "symbol" || temp == "rule" || temp == "diacritic" || temp == "root" || temp == "all") {
                class_flag += " synhi-keyword";
                add_textarea_span(element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }

            if(temp == "=>" || temp == "/" || temp == "_" || temp == "*" || temp == "$") {
                class_flag += " synhi-operator";
                add_textarea_span(element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }

            if(symbols.includes(temp) && type_flag != "rule") {
                class_flag += " synhi-symbol";
                add_textarea_span(element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }

            if(temp === "symbol" || temp === "diacritic") {
                class_flag += " synhi-symbol";
                symbol_flag = true;
            }

            if(temp === "feature" || temp === "switch") {
                type_flag = "feature"
            }

            if(temp === "rule") {
                type_flag = "rule";
            }
        }
        if(type_flag === "feature") {
            class_flag += " synhi-feature";
        }
        add_textarea_span(element, running, class_flag);
        element.appendChild(document.createElement("br"));
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
        try_compile(current_try_compilation_contents);
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
        for(var prop in response) {
            str += prop + " - " + response[prop][0] + ", line #" + response[prop][2];
            current_error_line = response[prop][2] - 1;
        }
        element.textContent = str;
    }
    if(prev_error_line != current_error_line) update_textarea(false);
    current_try_compilation_response_counter++;
    try_try_compile();
}

function add_textarea_span(element, content, type) {
    var temp = document.createElement("span");
    temp.className = "textarea-content " + type;
    temp.textContent = content;
    element.appendChild(temp);
}

function scroll_textarea() {
    var val = document.getElementById("program-textarea").scrollTop;
    var element = document.getElementById("textarea-renderer-container");
    element.style.top = (1.5-val) + "px";

    element = document.getElementById("line-numbers");
    element.style.top = (1.5-val) + "px";
}

document.getElementById("program-textarea").addEventListener("input", () => update_textarea(true));
document.getElementById("program-textarea").addEventListener("scroll", scroll_textarea);

document.getElementById('program-textarea').addEventListener('keydown', function(e) {
    if (e.key == 'Tab') {//based on https://stackoverflow.com/questions/6637341/use-tab-to-indent-in-textarea
        e.preventDefault();
        var start = this.selectionStart;
        var end = this.selectionEnd;
  
        this.value = this.value.substring(0, start) +
          "    " + this.value.substring(end);
  
        this.selectionStart =
          this.selectionEnd = start + 4;
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

        update_textarea(true)
    }
});

update_textarea(true);