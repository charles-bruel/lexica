function new_lexicon_state() {    
    return {
        names: ["POS", "Class", "Gender", "Word", "Translation", "Notes"],
        types: ["text", "text", "text", "text", "text", "note"],
        lens: [50, 55, 65, 200, 150, 100],
        num_rows: 20,
        cell_data: [],
    }
}

var current_lexicon_table_state = new_lexicon_state();

var lexicon_states = [current_lexicon_table_state];
var current_lexicon_id = 0;

const lexicon_root_element = document.getElementById("lexicon-table");
var lexicon_table_element;

function create_lexicon(row_count) {
    lexicon_table_element = document.createElement("table");
    lexicon_table_element.className = "lexicon-table";
    lexicon_root_element.appendChild(lexicon_table_element);

    var header_row = document.createElement("tr");
    lexicon_table_element.appendChild(header_row);

    var counter = 0;

    for(var i = 0;i < current_lexicon_table_state.names.length;i ++) {
        var header = document.createElement("th");
        header.className = "table-element table-header lexicon-header";
        var temp = current_lexicon_table_state.lens[i];
        header.style.width = temp + "px";
        counter += temp;
        header.innerText = current_lexicon_table_state.names[i];
        header_row.appendChild(header);
    }

    lexicon_table_element.style.width = counter + "px";

    for(var i = 0;i < current_lexicon_table_state.num_rows;i ++) {
        generate_lexicon_row(lexicon_table_element, i);
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

function generate_lexicon_row(table_element, row_id) {
    var row = document.createElement("tr");
    table_element.appendChild(row);

    for(var i = 0;i < current_lexicon_table_state.names.length;i ++) {
        var element = document.createElement("td");
        element.className = "table-element table-box lexicon-box";
        element.style.width = current_lexicon_table_state.lens[i] + "px";
        var box = document.createElement("input");
        box.style.width = "calc(100% - 4px)";
        box.style.padding = "0%";
        box.id = "lexicon-" + row_id + ":" + i;
        element.appendChild(box);
        row.appendChild(element);
    }
}

function save_lexicon_state() {
    current_lexicon_table_state.cell_data = [];
    for(var i = 0;i < current_lexicon_table_state.names.length;i ++) {
        current_lexicon_table_state.cell_data.push([]);
        for(var j = 0;j < current_lexicon_table_state.num_rows;j ++) {
            var element = document.getElementById("lexicon-" + j + ":" + i);
            current_lexicon_table_state.cell_data[i].push(element.value);
        }
    }
}

function delete_lexicon() {
    var container = document.getElementById("lexicon-table");
    container.replaceChildren();
}

function switch_lexicon_state(new_index) {
    save_lexicon_state();
    delete_lexicon();
    console.log(current_lexicon_table_state);
    lexicon_states[current_lexicon_id] = current_lexicon_table_state;
    current_lexicon_id = new_index;
    current_lexicon_table_state = lexicon_states[current_lexicon_id];
    create_lexicon();
}

create_lexicon();