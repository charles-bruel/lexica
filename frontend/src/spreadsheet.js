function new_spreadsheet_state() {
    return {
        num_rows: 60,
        num_columns: 26,
        row_height: 20,
        column_width: 100,
        row_header_width: 25,
        cell_data: [],
        underlying_cell_data: [],
        cell_style_classes: [],
    };
}

var current_spreadsheet_state = new_spreadsheet_state();
var current_spreadsheet_id = 0;
var spreadsheet_states = [current_spreadsheet_state];

function generate_cells() {
    var container = document.getElementById("spreadsheet-container");
    var countx = 0;
    var county = 0;
    container.style.gridTemplateRows = "";
    container.style.gridTemplateColumns = "";
    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++){
        if(i == 0) {
            container.style.gridTemplateRows += " " + current_spreadsheet_state.row_height + "px";
            countx += current_spreadsheet_state.row_height;
        }
        container.style.gridTemplateRows += " " + current_spreadsheet_state.row_height + "px";
        county += current_spreadsheet_state.row_height;
    }
    for(var i = 0;i < current_spreadsheet_state.num_columns;i ++){
        if(i == 0) {
            container.style.gridTemplateColumns += " " + current_spreadsheet_state.row_header_width + "px";
            countx += current_spreadsheet_state.row_header_width;
        }
        container.style.gridTemplateColumns += " " + current_spreadsheet_state.column_width + "px";
        countx += current_spreadsheet_state.column_width;
    }
    container.style.height = county + "px";
    container.style.width = countx + "px";
}

function create_header(posx, posy, text) {
    var element = document.createElement("div");
    element.className = "spreadsheet-header";
    element.innerText = text;
    element.style.gridColumnStart = posx;
    element.style.gridColumnStart = posx + 1;
    element.style.gridRowStart = posy;
    element.style.gridRowStart = posy + 1;
    return element;
}

//WTF javascript why is this nessecary
function setCharAt(str,index,chr) {
    if(index > str.length-1) return str;
    return str.substring(0,index) + chr + str.substring(index+1);
}

function convert_column_name(input) {
    const conversionA = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J'];
    const conversionB = ['K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z'];
    var temp = input.toString(26);
    for (var i = 0; i < temp.length; i++) {
        var charCode = temp.charCodeAt(i);
        if(charCode >= 97) {
            temp = setCharAt(temp, i, conversionB[charCode - 97]);
        } else {
            temp = setCharAt(temp, i, conversionA[charCode - 48]);
        }
    }
    return temp;
}

function populate_headers() {
    var container = document.getElementById("spreadsheet-container");
    container.appendChild(create_header(0, 0, ""));
    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++) {
        container.appendChild(create_header(0, i + 1, i + 1 + ""));
    }
    for(var i = 0;i < current_spreadsheet_state.num_columns;i ++) {
        container.appendChild(create_header(i + 1, 0, convert_column_name(i)));
    }
}

function populate_cells() {
    var container = document.getElementById("spreadsheet-container");
    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++) {
        for(var j = 0;j < current_spreadsheet_state.num_columns;j ++) {
            var element = document.createElement("input");
            element.id = "spreadsheet-" + i + ":" + j;
            element.style.gridColumnStart = j + 1;
            element.style.gridColumnStart = j + 2;
            element.style.gridRowStart = i + 1;
            element.style.gridRowStart = i + 2;
            container.appendChild(element);
        }
    }
}

function load_data() {
    if(current_spreadsheet_state.cell_data.length == 0) return;

    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++) {
        for(var j = 0;j < current_spreadsheet_state.num_columns;j ++) {
            var element = document.getElementById("spreadsheet-" + i + ":" + j)
            element.value = current_spreadsheet_state.cell_data[i][j];
            element.className = current_spreadsheet_state.cell_style_classes[i][j];
        }
    }
}

function create_spreadsheet() {
    generate_cells();
    populate_headers();
    populate_cells();
    load_data();
}

function save_spreadsheet_state() {
    current_spreadsheet_state.cell_data = [];
    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++) {
        current_spreadsheet_state.cell_data.push([]);
        current_spreadsheet_state.cell_style_classes.push([]);
        for(var j = 0;j < current_spreadsheet_state.num_columns;j ++) {
            var element = document.getElementById("spreadsheet-" + i + ":" + j);
            current_spreadsheet_state.cell_data[i].push(element.value);
            current_spreadsheet_state.cell_style_classes[i].push(element.className);
        }
    }
}

function delete_spreadsheet() {
    var container = document.getElementById("spreadsheet-container");
    container.replaceChildren();
}

function switch_spreadsheet_state(new_index) {
    save_spreadsheet_state();
    delete_spreadsheet();
    spreadsheet_states[current_spreadsheet_id] = current_spreadsheet_state;
    current_spreadsheet_id = new_index;
    current_spreadsheet_state = spreadsheet_states[current_spreadsheet_id];
    create_spreadsheet();
}

create_spreadsheet();


document.body.addEventListener("keyup", function(event) {
    if (event.key === "Enter") {
        document.activeElement.blur();
    }
});