var table_sources = {};
var table_states = {};
var current_table_id = 0;

const table_root_element = document.getElementById("lexicon-table");
var table_table_element;

function handle_table_column_resize(entries) {
    for(const entry of entries) {
        if(entry.target.parentNode.id.startsWith("lexicon-header-")) {
            var size = entry.borderBoxSize[0].inlineSize;
            entry.target.parentNode.style.width = size + "px";
        }
    }
}

function create_table_display() {
    table_table_element = document.createElement("table");
    table_table_element.className = "lexicon-table";
    table_root_element.appendChild(table_table_element);

    var header_row = document.createElement("tr");
    table_table_element.appendChild(header_row);

    var counter = 0;

    const observer = new ResizeObserver((entries) => { handle_table_column_resize(entries); });
    for(var i = 0;i < current_lexicon_table_state.names.length;i ++) {
        var header = document.createElement("th");
        var text_container = document.createElement("div");
        text_container.className = "table-element table-header lexicon-header";
        var temp = current_lexicon_table_state.lens[i];
        header.style.width = temp + "px";
        header.id = "lexicon-header-" + i;
        counter += temp;
        text_container.innerText = current_lexicon_table_state.names[i];
        header_row.appendChild(header);
        header.appendChild(text_container);
        observer.observe(text_container);
    }

    table_table_element.style.width = counter + "px";

    for(var i = 0;i < current_lexicon_table_state.num_rows;i ++) {
        generate_table_row(table_table_element, i);
    }
    
    if(current_lexicon_table_state.cell_data.length == 0) {
        return;
    }

    for(var i = 0; i < current_lexicon_table_state.cell_data.length;i ++) {
        var elems = current_lexicon_table_state.cell_data[i];
        for(var j = 0;j < elems.length;j ++) {
            var element = document.getElementById("lexicon-" + j + ":" + i);
            element.value = elems[j];
        }
    }
}

function generate_table_row(table_element, row_id) {
    var row = document.createElement("tr");
    table_element.appendChild(row);

    for(var i = 0;i < current_lexicon_table_state.names.length;i ++) {
        var element = document.createElement("td");
        element.className = "table-element table-box lexicon-box lexicon-box-" + current_lexicon_table_state.names[i];;
        element.style.width = current_lexicon_table_state.lens[i] + "px";
        var box = document.createElement("input");
        box.style.width = "calc(100% - 4px)";
        box.style.padding = "0%";
        box.id = "lexicon-" + row_id + ":" + i;
        element.appendChild(box);
        row.appendChild(element);
    }
}

function save_table_state() {
    // Check if we're editing the textarea and save it if we are
    if(document.getElementById("lexicon-textarea").style.display == "block") {
        table_sources[current_table_id] = document.getElementById("lexicon-textarea").value;
    }
}

function delete_table_display() {
    var container = document.getElementById("lexicon-table");
    container.replaceChildren();
}

function switch_table(new_index) {
    // save_lexicon_state();
    hide_blank_table_elements()
    delete_table_display();
    if(new_index in table_states) {
        if(new_index != current_table_id && current_table_id in table_states) {
            table_states[current_table_id] = current_lexicon_table_state;
        }
        current_table_id = new_index;
        current_lexicon_table_state = table_states[current_table_id];
        create_table_display();
    } else {
        current_table_id = new_index;
        show_blank_table_elements();
    }
}

function compile_tables() {
    // We've just loaded a save, so we have a bunch of table sources but
    // they aren't loaded yet. We need to load them now.
    for(let id in table_sources) {
        post_message({ LoadTable: { contents: table_sources[id] }});
    }

    // Rebuild all tables
    post_message({ RebuildTables: { start_index: 0 }});
}

function load_table_table(table) {
    console.log(table);

    id = table.id;
    
    names = []
    lens = []
    for(let column of table.table_descriptor.column_descriptors) {
        names.push(column.name);
        lens.push(column.column_display_width)
    }

    cell_data = []
    for(let i = 0;i < table.table_descriptor.column_descriptors.length;i ++) {
        cell_data.push([]);
    }

    for(let row of table.table_rows) {
        if(Object.hasOwn(row, "PopulatedTableRow")) {
            column = 0;
            for(let elem of row.PopulatedTableRow.contents) {
                cell_data[column].push(elem);
                column++;
            }
        } else {
            // TODO: Proper error message
            // alert("issue")
        }
    }

    table_states[id] = {
        names: names,
        lens: lens,
        num_rows: cell_data[0].length,
        cell_data: cell_data,
    }

    if(id == current_table_id) {
        switch_table(current_table_id);
    }
}

function show_blank_table_elements() {
    document.getElementById("lexicon-blank-elements").style.display = "block";
    // Also hide the edit button
    document.getElementById("lexicon-button-edit").style.display = "none";
}

function hide_blank_table_elements() {
    document.getElementById("lexicon-blank-elements").style.display = "none";
    // Also show the edit button
    document.getElementById("lexicon-button-edit").style.display = "block";
}

function on_table_selector_change() {
    var id = +document.getElementById("lexicon-table-select").value;
    switch_table(id);
}

function save_and_send_table() {
    var id = +document.getElementById("lexicon-table-select").value;
    // Get textarea value to send
    var text = document.getElementById("lexicon-textarea").value;
    // Send message to backend
    post_message({ LoadTable: { contents: text }});

    exit_table_edit_mode();
    // Send rebuild message
    post_message({ RebuildTables: { start_index: id }});
}

function create_new_table() {
    var id = +document.getElementById("lexicon-table-select").value;
    // If we already have contents for this table, it's been created
    // and we're waiting on the response from the server, so we don't
    // make anything new
    if(id in table_sources) {
        return;
    }
    table_sources[id] = `${id}\nCOLUMN1\nSTRING\nfoo`;
    // Update backend with table contents
    post_message({ LoadTable: { contents: table_sources[id] }});
}

function enter_table_edit_mode() {
    // Show save button
    document.getElementById("lexicon-button-save").style.display = "block";
    // Hide edit button
    document.getElementById("lexicon-button-edit").style.display = "none";
    // Show textarea
    document.getElementById("lexicon-textarea").style.display = "block";
    // Hide regular table display
    document.getElementById("lexicon-table").style.display = "none";
    // Update textarea contents
    document.getElementById("lexicon-textarea").value = table_sources[current_table_id];
}

function exit_table_edit_mode() {
    // Hide save button
    document.getElementById("lexicon-button-save").style.display = "none";
    // Show edit button
    document.getElementById("lexicon-button-edit").style.display = "block";
    // Hide textarea
    document.getElementById("lexicon-textarea").style.display = "none";
    // Show regular table display
    document.getElementById("lexicon-table").style.display = "block";
    // Save textarea contents
    table_sources[current_table_id] = document.getElementById("lexicon-textarea").value;
}

document.getElementById("lexicon-table-select").addEventListener('change', on_table_selector_change);
document.getElementById("lexicon-create-new").addEventListener('mousedown', create_new_table);
document.getElementById("lexicon-button-save").addEventListener('mousedown', save_and_send_table);
document.getElementById("lexicon-button-edit").addEventListener('mousedown', enter_table_edit_mode);