
var lexicon_table_layout = {
    names: ["POS", "Class", "Gender", "Word", "Translation", "Notes"],
    types: ["text", "text", "text", "text", "text", "note"],
    lens: [50, 55, 65, 200, 150, 100]
};

const lexicon_root_element = document.getElementById("lexicon-table");
var lexicon_table_element;
function generate_lexicon(row_count) {
    lexicon_table_element = document.createElement("table");
    lexicon_table_element.className = "lexicon-table";
    lexicon_root_element.appendChild(lexicon_table_element);

    var header_row = document.createElement("tr");
    lexicon_table_element.appendChild(header_row);

    var counter = 0;

    for(var i = 0;i < lexicon_table_layout.names.length;i ++) {
        var header = document.createElement("th");
        header.className = "table-element table-header lexicon-header";
        var temp = lexicon_table_layout.lens[i];
        header.style.width = temp + "px";
        counter += temp;
        header.innerText = lexicon_table_layout.names[i];
        header_row.appendChild(header);
    }

    lexicon_table_element.style.width = counter + "px";

    for(var i = 0;i < row_count;i ++) {
        generate_lexicon_row(lexicon_table_element, i);
    }    
}

function generate_lexicon_row(table_element, row_id) {
    var row = document.createElement("tr");
    table_element.appendChild(row);

    for(var i = 0;i < lexicon_table_layout.names.length;i ++) {
        var element = document.createElement("td");
        element.className = "table-element table-box lexicon-box";
        element.style.width = lexicon_table_layout.lens[i] + "px";
        var box = document.createElement("input");
        box.style.width = "calc(100% - 4px)";
        box.style.padding = "0%";
        box.id = "lexicon-" + row_id + ":" + i;
        element.appendChild(box);
        row.appendChild(element);
    }
}

generate_lexicon(100);