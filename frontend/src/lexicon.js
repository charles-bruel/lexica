function new_lexicon_state() {    
    return {
        names: ["POS", "Class", "Gender", "Word", "Translation", "Notes"],
        lens: [50, 55, 65, 200, 150, 100],
        num_rows: 20,
        cell_data: [],
    }
}

var current_lexicon_table_state = new_lexicon_state();

var lexicon_states = { 0: current_lexicon_table_state};
var current_lexicon_id = 0;

const lexicon_root_element = document.getElementById("lexicon-table");
var lexicon_table_element;

function handle_lexicon_column_resize(entries) {
    for(const entry of entries) {
        if(entry.target.parentNode.id.startsWith("lexicon-header-")) {
            var size = entry.borderBoxSize[0].inlineSize;
            entry.target.parentNode.style.width = size + "px";
        }
    }
}

function create_lexicon() {
    lexicon_table_element = document.createElement("table");
    lexicon_table_element.className = "lexicon-table";
    lexicon_root_element.appendChild(lexicon_table_element);

    var header_row = document.createElement("tr");
    lexicon_table_element.appendChild(header_row);

    var counter = 0;

    const observer = new ResizeObserver((entries) => { handle_lexicon_column_resize(entries); });
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

// function save_lexicon_state() {
//     current_lexicon_table_state.cell_data = [];
//     for(var i = 0;i < current_lexicon_table_state.names.length;i ++) {
//         current_lexicon_table_state.cell_data.push([]);
//         for(var j = 0;j < current_lexicon_table_state.num_rows;j ++) {
//             var element = document.getElementById("lexicon-" + j + ":" + i);
//             current_lexicon_table_state.cell_data[i].push(element.value);
//         }
//     }

//     lexicon_states[current_lexicon_id] = current_lexicon_table_state;
// }

function delete_lexicon() {
    var container = document.getElementById("lexicon-table");
    container.replaceChildren();
}

function switch_lexicon_state(new_index) {
    // save_lexicon_state();
    delete_lexicon();
    if(new_index in lexicon_states) {
        lexicon_states[current_lexicon_id] = current_lexicon_table_state;
        current_lexicon_id = new_index;
        current_lexicon_table_state = lexicon_states[current_lexicon_id];
        create_lexicon();  
    } else {
        post_message({ LoadTable: { contents: "1\nPOS|WORD|TRANSLATION|INDEX\n[ROOT,NOUN,PRONOUN,VERB,ADJECTIVE,ADVERB,PARTICLE]|STRING|STRING|UINT\nROOT|ran|earth|0\n" }});
    }
}

function load_lexicon_table(table) {
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
            alert("issue")
        }
    }

    lexicon_states[id] = {
        names: names,
        lens: lens,
        num_rows: 20,
        cell_data: cell_data,
    }

    if(id == current_lexicon_id) {
        switch_lexicon_state(current_lexicon_id);
    }
}

create_lexicon();

function load_lexicon_program() {
    switch_lexicon_state(+document.getElementById("lexicon-table-select").value);
}

document.getElementById("lexicon-table-select").addEventListener('change', load_lexicon_program);