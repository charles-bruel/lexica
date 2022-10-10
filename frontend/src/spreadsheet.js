function new_spreadsheet_state() {
    return {
        num_rows: 30,
        num_columns: 24,
        row_height: 20,
        column_width: 50, // Default
        column_widths: [],
        row_header_width: 35,
        cell_data: [],
        cell_merge_data: [],
        underlying_cell_data: [],
        cell_style_classes: [],
    };
}

var current_spreadsheet_state = new_spreadsheet_state();
var current_spreadsheet_id = 0;
var spreadsheet_states = [current_spreadsheet_state];
var spreadsheet_container;

function generate_cells() {
    var container = document.getElementById("spreadsheet-container");
    var countx = 0;
    var county = 0;
    container.style.gridTemplateRows = "";
    container.style.gridTemplateColumns = "";
    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++){
        if(i == 0) {
            container.style.gridTemplateRows += " " + current_spreadsheet_state.row_height + "px";
            county += current_spreadsheet_state.row_height;
        }
        container.style.gridTemplateRows += " " + current_spreadsheet_state.row_height + "px";
        county += current_spreadsheet_state.row_height;
        if(i == current_spreadsheet_state.num_rows - 1) {
            container.style.gridTemplateRows += " " + current_spreadsheet_state.row_height + "px";
            county += current_spreadsheet_state.row_height;
        }
    }

    if(current_spreadsheet_state.column_widths.length == 0) {
        current_spreadsheet_state.column_widths = Array(current_spreadsheet_state.num_columns).fill(current_spreadsheet_state.column_width);
    }

    for(var i = 0;i < current_spreadsheet_state.num_columns;i ++){
        if(i == 0) {
            container.style.gridTemplateColumns += " " + current_spreadsheet_state.row_header_width + "px";
            countx += current_spreadsheet_state.row_header_width;
        }
        container.style.gridTemplateColumns += " " + current_spreadsheet_state.column_widths[i] + "px";
        countx += current_spreadsheet_state.column_widths[i];
        if(i == current_spreadsheet_state.num_columns - 1) {
            container.style.gridTemplateColumns += " " + current_spreadsheet_state.row_header_width + "px";
            countx += current_spreadsheet_state.row_header_width;
        }
    }
    container.style.height = county + "px";
    container.style.width = countx + "px";

    spreadsheet_container = container;
}

function create_header(posx, posy, text) {
    var element = document.createElement("div");
    element.className = "spreadsheet-header unselectable";
    if(posx != 0) {
        element.className += " spreadsheet-column-header";
        element.id = "spreadsheet-header-col-" + posx;
    } else {
        element.id = "spreadsheet-header-row-" + posy;
    }
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
    if(input == 0) return "A";

    const table = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z']

    var result = [];
    var iter = 0;
    while (input > 0) {
        var b = 27;
        if(iter == 0) { b = 26; }
        var temp = input % b;
        if(iter != 0) { temp -= 1; }
        var char = table[temp];
        result.splice(0, 0, char);
        input = Math.floor(input / b);
        iter ++;
    }

    return result.join("");
}

function handle_column_resize(entries) {
    for(const entry of entries) {
        if(entry.target.id.startsWith("spreadsheet-header-col-")) {
            id = entry.target.id.substring(23) - 0;
            var size = entry.borderBoxSize[0].inlineSize;
            var elems = spreadsheet_container.style.gridTemplateColumns.split(" ");
            elems[id] = size + "px";
            spreadsheet_container.style.gridTemplateColumns = elems.join(" ");
        }
    }
}

function add_row_handler(button_ref) {
    current_spreadsheet_state.num_rows++;

    var elems = spreadsheet_container.style.gridTemplateRows.split(" ");
    var index = elems.length - 2;
    elems.splice(index, 0, current_spreadsheet_state.row_height + "px");
    spreadsheet_container.style.gridTemplateRows = elems.join(" ");
    button_ref.style.gridRowStart = parseInt(button_ref.style.gridRowStart) + 1;
    spreadsheet_container.parentNode.scrollBy(0, current_spreadsheet_state.row_height);

    var element = create_header(0, index + 1, index + 1 + "");
    spreadsheet_container.appendChild(element);

    for(var i = 0;i < current_spreadsheet_state.num_columns;i ++) {
        var element = document.createElement("input");
        element.id = "spreadsheet-" + index + ":" + i;
        element.style.gridColumnStart = i + 2;
        element.style.gridRowStart = index + 2;
        spreadsheet_container.appendChild(element);
    }
}

function add_column_handler(button_ref) {
    current_spreadsheet_state.num_columns++;

    var elems = spreadsheet_container.style.gridTemplateColumns.split(" ");
    var index = elems.length - 2;
    elems.splice(index, 0, current_spreadsheet_state.column_width + "px");
    spreadsheet_container.style.gridTemplateColumns = elems.join(" ");
    button_ref.style.gridColumnStart = parseInt(button_ref.style.gridColumnStart) + 1;
    spreadsheet_container.parentNode.scrollBy(current_spreadsheet_state.column_width, 0);

    const observer = new ResizeObserver((entries) => { handle_column_resize(entries); });
    var element = create_header(index + 1, 0, convert_column_name(index));
    spreadsheet_container.appendChild(element);
    observer.observe(element);

    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++) {
        var element = document.createElement("input");
        element.id = "spreadsheet-" + i + ":" + index;
        element.style.gridColumnStart = index + 2;
        element.style.gridRowStart = i + 2;
        spreadsheet_container.appendChild(element);
    }
}

function create_expansion_button_rows(posy) {
    var element = document.createElement("button");
    element.className = "spreadsheet-header spreadsheet-expansion-button";
    element.id = "spreadsheet-expansion-row";
    element.innerText = "+";
    element.style.gridColumnStart = 0;
    element.style.gridColumnStart = 1;
    element.style.gridRowStart = posy;
    element.style.gridRowStart = posy + 1;
    element.addEventListener("mousedown", function() { add_row_handler(element); });
    return element;
}

function create_expansion_button_columns(posx) {
    var element = document.createElement("button");
    element.className = "spreadsheet-header spreadsheet-expansion-button";
    element.id = "spreadsheet-expansion-column";
    element.innerText = "+";
    element.style.gridColumnStart = posx;
    element.style.gridColumnStart = posx + 1;
    element.style.gridRowStart = 0;
    element.style.gridRowStart = 1;
    element.addEventListener("mousedown", function() { add_column_handler(element); });
    return element;
}

function populate_headers() {
    var container = document.getElementById("spreadsheet-container");
    const observer = new ResizeObserver((entries) => { handle_column_resize(entries); });
    container.appendChild(create_header(0, 0, ""));
    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++) {
        container.appendChild(create_header(0, i + 1, i + 1 + ""));
    }
    for(var i = 0;i < current_spreadsheet_state.num_columns;i ++) {
        var element = create_header(i + 1, 0, convert_column_name(i));
        container.appendChild(element);
        observer.observe(element);
    }
    container.appendChild(create_expansion_button_rows(current_spreadsheet_state.num_rows + 1));
    container.appendChild(create_expansion_button_columns(current_spreadsheet_state.num_columns + 1));
}

function populate_cells() {
    var container = document.getElementById("spreadsheet-container");
    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++) {
        for(var j = 0;j < current_spreadsheet_state.num_columns;j ++) {
            var element = document.createElement("input");
            element.id = "spreadsheet-" + i + ":" + j;
            element.style.gridColumnStart = j + 2;
            element.style.gridRowStart = i + 2;
            container.appendChild(element);
        }
    }
}

function load_data() {
    if(current_spreadsheet_state.cell_data.length == 0) return;

    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++) {
        for(var j = 0;j < current_spreadsheet_state.num_columns;j ++) {
            var element = document.getElementById("spreadsheet-" + i + ":" + j);
            element.value = current_spreadsheet_state.cell_data[i][j];
            element.className = current_spreadsheet_state.cell_style_classes[i][j];
        }
    }

    for(var i = 0;i < current_spreadsheet_state.cell_merge_data.length;i ++) {
        var merge = current_spreadsheet_state.cell_merge_data[i];
        merge_cells(merge.x1, merge.x2, merge.y1, merge.y2, false);
    }
}

function merge_cells(x1, x2, y1, y2, add) {
    var fx1, fx2, fy1, fy2

    fx1 = Math.min(x1, x2);
    fx2 = Math.max(x1, x2);
    fy1 = Math.min(y1, y2);
    fy2 = Math.max(y1, y2);

    var strings = [];
    for(var i = fx1;i <= fx2;i ++) {
        for(var j = fy1;j <= fy2;j ++) {
            var element = document.getElementById("spreadsheet-" + j + ":" + i);
            if(element != null) {
                if(element.value != "") strings.push(element.value);
                spreadsheet_container.removeChild(element);
            }
        }
    }
    var element;
    if(y1 == y2) {
        element = document.createElement("input");
    } else {
        element = document.createElement("textarea");
        element.spellcheck = false;
    }
    element.id = "spreadsheet-" + fy1 + ":" + fx1;
    element.style.gridColumnStart = fx1 + 2;
    element.style.gridColumnEnd = fx2 + 3;
    element.style.gridRowStart = fy1 + 2;
    element.style.gridRowEnd = fy2 + 3;
    element.value = strings.join(" ");
    spreadsheet_container.appendChild(element);

    if(add) current_spreadsheet_state.cell_merge_data.push({ x1: fx1, x2: fx2, y1: fy1, y2: fy2 })
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
            if(element != null) {
                current_spreadsheet_state.cell_data[i].push(element.value);
                current_spreadsheet_state.cell_style_classes[i].push(element.className);
            } else {
                current_spreadsheet_state.cell_data[i].push("");
                current_spreadsheet_state.cell_style_classes[i].push("");
            }
        }
    }

    current_spreadsheet_state.column_widths = [];
    for(var i = 0;i < current_spreadsheet_state.num_columns;i ++) {
        var elems = spreadsheet_container.style.gridTemplateColumns.split(" ");
        var temp = parseInt(elems[i + 1].replace("px", ""));
        current_spreadsheet_state.column_widths.push(temp);
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