var programs = {};

function update_textarea() {
    var area = document.getElementById("program-textarea");
    var element = document.getElementById("textarea-renderer");
    var temp = area.value.replace("\t", "    ");
    var lines = temp.split("\n");
    element.replaceChildren();
    for(var i = 0;i < lines.length;i ++) {
        var temp = document.createElement("span");
        temp.className = "textarea-content";
        temp.textContent = lines[i];
        element.appendChild(temp);
        element.appendChild(document.createElement("br"));
    }
}

function scroll_textarea() {
    var val = document.getElementById("program-textarea").scrollTop;
    var element = document.getElementById("textarea-renderer");
    element.style.top = -val + "px";
}

document.getElementById("program-textarea").addEventListener("input", update_textarea);
document.getElementById("program-textarea").addEventListener("scroll", scroll_textarea);