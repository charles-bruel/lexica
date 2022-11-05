function generate_table_of_contents() {
    var body = document.body.children;
    var element = document.getElementById("table-of-contents-container");
    var top_level_ol = document.createElement("ol");
    var running_ol;
    var last_element;
    element.appendChild(top_level_ol);
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
            top_level_ol.appendChild(temp);
            last_element = temp;

            prev_element_type = "H2";
        } else if (child.tagName == "H3") {
            if(prev_element_type != "H3") {
                running_ol = document.createElement("ol");
                running_ol.style.marginLeft = "0.0in"
                last_element.append(running_ol);
            }
            var id = child.textContent.replaceAll(" ", "-").toLowerCase();
            child.id = id;
            var temp = document.createElement("li");
            var temp2 = document.createElement("a");
            temp2.href = "#" + id;
            temp2.textContent = child.textContent;
            temp.appendChild(temp2);
            running_ol.appendChild(temp);
            last_element = temp;

            prev_element_type = "H3";
        }
    }
}

generate_table_of_contents();

function generate_indent_lines() {
    var arr = Array.from(document.body.children);
    for(var i = 0;i < arr.length;i ++) {
        var tn = arr[i].tagName;
        if(tn == "H1" || tn == "H2" || tn == "H3") generate_indent_line(arr[i], arr, i);
    }
}

function generate_indent_line(element, arr, index) {
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
    if(element.tagName == "H1") div.style.left = "0.325in";
    if(element.tagName == "H2") div.style.left = "0.525in";
    if(element.tagName == "H3") div.style.left = "0.725in";

    element.parentNode.appendChild(div);
}

generate_indent_lines();