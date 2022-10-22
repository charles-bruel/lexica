var programs = {};

function update_textarea() {
    var area = document.getElementById("program-textarea");
    var element = document.getElementById("textarea-renderer-container");
    var temp = area.value.replace("\t", "    ");
    var lines = temp.split("\n");
    element.replaceChildren();
    var symbols = [];
    for(var i = 0;i < lines.length;i ++) {
        var type_flag = "";
        var whitespcae_flag = 0;
        var class_flag = "";
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
                class_flag = "";
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
                class_flag = "";
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
                class_flag = "";
            }

            var end_flag =  j == lines[i].length - 1 || /\s/.test(lines[i][j + 1]);
            if(!end_flag) continue;

            if(temp == "feature_def" || temp == "symbols" || temp == "rules" || temp == "rule" || temp == "diacritics" || temp == "end") {
                class_flag += " synhi-top-level";
                add_textarea_span(element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = "";
            }

            if(temp == "feature" || temp == "switch" || temp == "symbol" || temp == "rule" || temp == "diacritic" || temp == "root" || temp == "all") {
                class_flag += " synhi-keyword";
                add_textarea_span(element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = "";
            }

            if(temp == "=>" || temp == "/" || temp == "_" || temp == "*" || temp == "$") {
                class_flag += " synhi-operator";
                add_textarea_span(element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = "";
            }

            if(symbols.includes(temp) && type_flag != "rule") {
                class_flag += " synhi-symbol";
                add_textarea_span(element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = "";
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
    element.style.top = (3-val) + "px";
}

document.getElementById("program-textarea").addEventListener("input", update_textarea);
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

        update_textarea()
    }
  });