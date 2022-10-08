function handle_basic_button(e, clazz) {
    e.preventDefault();

    var element = document.activeElement;
    if (element == null) return;

    if(element.id == "") return;
    if(element.id.startsWith("spreadsheet") || element.id.startsWith("lexicon")) {
        if(element.classList.contains(clazz)){
            element.classList.remove(clazz);
        } else {
            element.classList.add(clazz);
        }
    }
}

function handle_spreadsheet_button(e, clazz) {
    e.preventDefault();

    var element = document.activeElement;
    if (element == null) return;

    if(element.id == "") return;
    if(element.id.startsWith("spreadsheet")) {
        if(element.classList.contains(clazz)){
            element.classList.remove(clazz);
        } else {
            element.classList.add(clazz);
        }
    }
}

document.getElementById("format-bold").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-bold"); });
document.getElementById("format-italic").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-italic"); });
document.getElementById("format-underline").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-underline"); });
document.getElementById("format-strikethrough").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-strikethrough"); });
document.getElementById("format-bg").addEventListener("mousedown", function(e) { handle_spreadsheet_button(e, "formatting-bg"); });
