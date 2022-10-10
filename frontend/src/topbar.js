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

var topbar_merge_button_state = false;
var document_last_active_element = -1;
var selected_elements_for_merge = [];
function handle_merge(e, element) {
    if(topbar_merge_button_state) {
        topbar_merge_button_state = false;

        if(selected_elements_for_merge.length >= 2) {            
            var x1 = selected_elements_for_merge[0].style.gridColumnStart;
            var y1 = selected_elements_for_merge[0].style.gridRowStart;
            var x2 = selected_elements_for_merge[1].style.gridColumnStart;
            var y2 = selected_elements_for_merge[1].style.gridRowStart;
            merge_cells(x1 - 2, x2 - 2, y1 - 2, y2 - 2, true);
        }

        element.classList.remove("selected");
        for(const x of selected_elements_for_merge) {
            x.classList.remove("selected");
        }
        selected_elements_for_merge = [];
    } else {
        topbar_merge_button_state = true;
        element.classList.add("selected");
    }
}

function detectBlur() {}

function detectFocus() {
    if(!topbar_merge_button_state) return;
    if(document.activeElement !== document_last_active_element) {
        document_last_active_element = document.activeElement;

        if(document.activeElement.id.startsWith("spreadsheet-")) {
            if(!document.activeElement.classList.contains("selected")) {
                if(selected_elements_for_merge.length >= 2) {
                    selected_elements_for_merge[1].classList.remove("selected");
                    selected_elements_for_merge[1] = document.activeElement;
                } else {
                    selected_elements_for_merge.push(document.activeElement);
                }

                document.activeElement.classList.add("selected");
            } else {
                document.activeElement.classList.remove("selected");

                selected_elements_for_merge.splice(selected_elements_for_merge.indexOf(document.activeElement), 1);
            }
        }
    }
}

function attachEvents() {
    window.addEventListener ? window.addEventListener('focus', detectFocus, true) : window.attachEvent('onfocusout', detectFocus);  
    window.addEventListener ? window.addEventListener('blur', detectBlur, true) : window.attachEvent('onblur', detectBlur);
}

document.getElementById("format-bold").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-bold"); });
document.getElementById("format-italic").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-italic"); });
document.getElementById("format-underline").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-underline"); });
document.getElementById("format-strikethrough").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-strikethrough"); });
document.getElementById("format-bg").addEventListener("mousedown", function(e) { handle_spreadsheet_button(e, "formatting-bg"); });
document.getElementById("format-merge").addEventListener("mousedown", function(e) { handle_merge(e, document.getElementById("format-merge")); });

attachEvents();