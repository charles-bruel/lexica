function generate_table_of_contents() {//This is super ugly and scales horribly
    var body = document.body.children;
    var element = document.getElementById("table-of-contents-container");
    var top_level_list = document.createElement("ul");
    var running_list;
    var running_list_2;
    var last_element_h2;
    var last_element_h3;
    element.appendChild(top_level_list);
    var prev_element_type = "";
    for(var child of body) {
        if(child.tagName == "H2") {
            var id = child.textContent.replaceAll(" ", "-").toLowerCase();
            child.id = id;
            var temp = document.createElement("li");
            var temp2 = document.createElement("a");
            temp2.href = "#" + id;
            temp2.textContent = child.textContent;
            temp.appendChild(temp2);
            top_level_list.appendChild(temp);
            last_element_h2 = temp;

            prev_element_type = "H2";
        } else if (child.tagName == "H3") {
            if(prev_element_type != "H3") {
                running_list = document.createElement("ul");
                running_list.style.marginLeft = "0.0in"
                last_element_h2.append(running_list);
            }
            var id = child.textContent.replaceAll(" ", "-").toLowerCase();
            child.id = id;
            var temp = document.createElement("li");
            var temp2 = document.createElement("a");
            temp2.href = "#" + id;
            temp2.textContent = child.textContent;
            temp.appendChild(temp2);
            running_list.appendChild(temp);
            last_element_h3 = temp;

            prev_element_type = "H3";
        } else if (child.tagName == "H4") {
            if(prev_element_type != "H4") {
                running_list_2 = document.createElement("ul");
                running_list_2.style.marginLeft = "0.0in"
                last_element_h3.append(running_list_2);
            }
            var id = child.textContent.replaceAll(" ", "-").toLowerCase();
            child.id = id;
            var temp = document.createElement("li");
            var temp2 = document.createElement("a");
            temp2.href = "#" + id;
            temp2.textContent = child.textContent;
            temp.appendChild(temp2);
            running_list_2.appendChild(temp);

            prev_element_type = "H4";
        }
    }
}

function generate_indent_lines() {
    var element = document.getElementById("indent-lines");
    element.replaceChildren();
    var arr = Array.from(document.body.children);
    for(var i = 0;i < arr.length;i ++) {
        var tn = arr[i].tagName;
        if(tn == "H1" || tn == "H2" || tn == "H3") generate_indent_line(arr[i], arr, i, element);
    }
}

function generate_indent_line(element, arr, index, parent_elem) {
    var following_element;
    var flag = true;
    for(var i = index + 1;i < arr.length;i ++) {
        following_element = arr[i];
        var flag2 = false;
        if(arr[i].tagName == element.tagName) flag2 = true;
        if(element.tagName == "H3" && (arr[i].tagName == "H2" || arr[i].tagName == "H1")) flag2 = true;
        if(element.tagName == "H2" && arr[i].tagName == "H1") flag2 = true;
        if(flag2) {
            flag = false;
            break;
        }
    }

    var div = document.createElement("div");
    div.style.position = "absolute";
    var top = element.offsetTop + element.getBoundingClientRect().height;
    var end = following_element.offsetTop + (flag ? following_element.getBoundingClientRect().height : 0);
    div.style.top = top + "px";
    div.style.backgroundColor = "lavender";
    div.style.width = "1px";
    div.style.height = (end - top) + "px";
    if(element.tagName == "H1") div.style.left = "2.225in";
    if(element.tagName == "H2") div.style.left = "2.425in";
    if(element.tagName == "H3") div.style.left = "2.625in";

    parent_elem.appendChild(div);
}

function syntax_highlighter(parent_element, text_content, symbols) {
    var temp = text_content.replace("\t", "    ");
    var lines = temp.split("\n");
    parent_element.replaceChildren();
    for(var i = 0;i < lines.length;i ++) {
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
                add_textarea_span(parent_element, running, class_flag);
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
                add_textarea_span(parent_element, running, class_flag);
                running = "";
                whitespcae_flag = 0;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }
            if(char === "#"){
                add_textarea_span(parent_element, lines[i].substring(j), "synhi-comment");
                break;
            }
            
            running += char;
            var temp = running.trimStart();

            if(temp == "(" || temp == ")" || temp == "[" || temp == "]" || temp == "{" || temp == "}") {
                class_flag += " synhi-paren";
                add_textarea_span(parent_element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }

            var end_flag =  j == lines[i].length - 1 || /\s/.test(lines[i][j + 1]);
            if(!end_flag) continue;

            if(temp == "feature_def" || temp == "symbols" || temp == "rules" || temp == "rule" || temp == "diacritics" || temp == "end") {
                class_flag += " synhi-top-level";
                add_textarea_span(parent_element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }

            if(temp == "feature" || temp == "switch" || temp == "symbol" || temp == "rule" || temp == "diacritic" || temp == "root" || temp == "all" || temp == "sub"
                || temp == "subx" || temp == "call" || temp == "label" || temp == "jmp" || temp == "mod" || temp == "flag" || temp == "!mod" || temp == "!flag") {
                    //There is nothing wrong with this please send help
                class_flag += " synhi-keyword";
                add_textarea_span(parent_element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }

            if(temp == "=>" || temp == "/" || temp == "//" || temp == "_" || temp == "*" || temp == "$") {
                class_flag += " synhi-operator";
                add_textarea_span(parent_element, running, class_flag);
                running = "";
                whitespcae_flag = false;
                class_flag = (i === current_error_line ? "synhi-err" : "");
            }

            if(symbols.includes(temp) && type_flag != "rule") {
                class_flag += " synhi-symbol";
                add_textarea_span(parent_element, running, class_flag);
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
        add_textarea_span(parent_element, running, class_flag);
        parent_element.appendChild(document.createElement("br"));
    }
}

current_error_line = -1;

function add_textarea_span(element, content, type) {
    var temp = document.createElement("span");
    temp.className = "textarea-content " + type;
    temp.textContent = content;
    element.appendChild(temp);
}

function format_code_blocks() {
    var blocks = document.getElementsByTagName("pre");
    for(var i = 0; i < blocks.length; i++) {
        var temp = document.createElement("div");
        temp.className = "code";
        blocks[i].parentNode.prepend(temp);
        syntax_highlighter(temp, blocks[i].textContent, blocks[i].className.split(" "));
        blocks[i].parentNode.removeChild(blocks[i]);
        i--;
    }
}