var programs = {};

function update_textarea() {
    var area = document.getElementById("program-textarea");
    var element = document.getElementById("textarea-renderer");
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

            if(symbols.includes(temp)) {
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
    var element = document.getElementById("textarea-renderer");
    element.style.top = -val + "px";
}

document.getElementById("program-textarea").addEventListener("input", update_textarea);
document.getElementById("program-textarea").addEventListener("scroll", scroll_textarea);