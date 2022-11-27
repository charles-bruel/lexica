function handle_basic_button(e, clazz) {
    e.preventDefault();

    var elements = get_elements_for_modification();
    if(elements.length === 0) return;
    var to_remove = elements[0].classList.contains(clazz);
    for(const element of elements) {
        if(element.classList.contains(clazz)){
            element.classList.remove(clazz);
        } else {
            element.classList.add(clazz);
        }
    }
}

function handle_align_button(e, clazz) {
    e.preventDefault();

    var elements = get_elements_for_modification();

    for(const element of elements) {
        element.classList.remove("formatting-left");
        element.classList.remove("formatting-right");
        element.classList.remove("formatting-center");
        element.classList.add(clazz);
    }
}

function handle_spreadsheet_button(e, clazz) {
    e.preventDefault();

    var elements = get_elements_for_modification();
    if(elements.length === 0) return;
    var to_remove = elements[0].classList.contains(clazz);
    for(const element of elements) {
        if(to_remove){
            element.classList.remove(clazz);
        } else {
            element.classList.add(clazz);
        }
    }
}

function handle_merge(e, element) {
    if(document.activeElement.classList.contains("merged")) {
        umerge_cells(document.activeElement);
    } else {
        var mini = Math.min(selection_base_pos.i, selection_extent_pos.i);
        var maxi = Math.max(selection_base_pos.i, selection_extent_pos.i);
        var minj = Math.min(selection_base_pos.j, selection_extent_pos.j);
        var maxj = Math.max(selection_base_pos.j, selection_extent_pos.j);

        merge_cells(minj, maxj, mini, maxi, true);
    }
    
}

document.getElementById("format-bold").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-bold"); });
document.getElementById("format-italic").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-italic"); });
document.getElementById("format-underline").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-underline"); });
document.getElementById("format-strikethrough").addEventListener("mousedown", function(e) { handle_basic_button(e, "formatting-strikethrough"); });
document.getElementById("format-bg").addEventListener("mousedown", function(e) { handle_spreadsheet_button(e, "formatting-bg"); });
document.getElementById("format-merge").addEventListener("mousedown", function(e) { handle_merge(e, document.getElementById("format-merge")); });
document.getElementById("format-right").addEventListener("mousedown", function(e) { handle_align_button(e, "formatting-right"); });
document.getElementById("format-center").addEventListener("mousedown", function(e) { handle_align_button(e, "formatting-center"); });
document.getElementById("format-left").addEventListener("mousedown", function(e) { handle_align_button(e, "formatting-left"); });

document.getElementById("programs-button").addEventListener("mousedown", function(e) { 
    var elem = document.getElementById("program-menu");
    if(elem.style.display == "") {
        elem.style.display = "none";
    } else {
        elem.style.display = "";
    }
});

document.getElementById("button-save").addEventListener("mousedown", function(e) { 
    post_message({SaveFile: {file_path: document.getElementById("save-file-location").value, data: get_state_for_save(), overwrite: false}}); 
});

document.getElementById("button-load").addEventListener("mousedown", function(e) { 
    post_message({LoadFile: {file_path: document.getElementById("save-file-location").value}}); 
});