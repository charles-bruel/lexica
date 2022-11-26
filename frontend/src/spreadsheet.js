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

var last_focused_cell = null;
var underyling_editor_mode = false;

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

function handle_spreadsheet_column_resize(entries) {
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

    const observer = new ResizeObserver((entries) => { handle_spreadsheet_column_resize(entries); });
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
    const observer = new ResizeObserver((entries) => { handle_spreadsheet_column_resize(entries); });
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

function handle_blur(element, i, j) {
    if(underyling_editor_mode) {
        current_spreadsheet_state.underlying_cell_data[i][j] = element.value
    }
    last_focused_cell = null;
    underyling_editor_mode = false;
    element.value = eval_spreadsheet_formula(current_spreadsheet_state.underlying_cell_data[i][j], { i: i, j:j });
    set_root_variable("--select-color", "blue");
}

function handle_focus(element, i, j, dbl) {
    if(dbl) {
        //Enter underlying value editor
        underyling_editor_mode = true;
        element.value = current_spreadsheet_state.underlying_cell_data[i][j];
        set_root_variable("--select-color", "green");
    } else {
        last_focused_cell = element;
    }
}

function populate_single_cell(container, i, j) {
    var element = document.createElement("input");
    element.id = "spreadsheet-" + i + ":" + j;
    element.className = "spreadsheet-cell";
    element.style.gridColumnStart = j + 2;
    element.style.gridRowStart = i + 2;
    // element.disabled = true;
    container.appendChild(element);
}

var selection_base_pos;
var selection_extent_pos;
var selection_base_element;
var selection_mode_active;
var selection_extents_element;

selection_extents_element = document.getElementById("spreadsheet-selection-extents");

function show_selection_extents() {
    var mini = Math.min(selection_base_pos.i, selection_extent_pos.i);
    var maxi = Math.max(selection_base_pos.i, selection_extent_pos.i);
    var minj = Math.min(selection_base_pos.j, selection_extent_pos.j);
    var maxj = Math.max(selection_base_pos.j, selection_extent_pos.j);

    selection_extents_element.style.gridRowStart = mini + 2;
    selection_extents_element.style.gridRowEnd = maxi + 3;
    selection_extents_element.style.gridColumnStart = minj + 2;
    selection_extents_element.style.gridColumnEnd = maxj + 3;

    console.log("foo");
}

document.addEventListener('mousemove', e => {
    if(!selection_mode_active) return;
    var element = document.elementFromPoint(e.clientX, e.clientY);
    if(element.className != "spreadsheet-cell") return;
    var id = element.id;
    id = id.substring(12);
    var nums = id.split(":");
    var i = parseInt(nums[0]);
    var j = parseInt(nums[1]);
    selection_extent_pos = { i: i, j: j };
    show_selection_extents();
});

document.addEventListener('mousedown', e => {
    if(selection_mode_active) return;
    var element = document.elementFromPoint(e.clientX, e.clientY);
    if(element.className != "spreadsheet-cell") return;
    var id = element.id;
    id = id.substring(12);
    var nums = id.split(":");
    var i = parseInt(nums[0]);
    var j = parseInt(nums[1]);
    selection_base_pos = { i: i, j: j };
    selection_base_element = element;
    selection_mode_active = true;
});

document.addEventListener('mouseup', e => {
    var element = document.elementFromPoint(e.clientX, e.clientY);
    if(element.className != "spreadsheet-cell") return;
    selection_mode_active = false;
});

function handle_click(e, element, i, j, dbl) {
    if(!dbl) {
        selection_base_pos = { i: i, j: j };
        selection_base_element = element;
        e.preventDefault();
    }
}

function handle_mouse_enter(e, element, i, j) {
    if(typeof selection_base_pos !== 'undefined' && selection_base_pos.i == i && selection_base_pos.j == j) {
        element.blur();
    }
    if(e.buttons === 1) {
        console.log(i + ", " + j);
        if(selection_base_element != null) {
            // console.log(selection_base_element === document.activeElement);
            selection_base_element.blur();
        }
        // selection_extent_pos = { i: i, j: j }
        // console.log({start: selection_base_pos, end: selection_extent_pos});
    }
}

function populate_cells() {
    var container = document.getElementById("spreadsheet-container");
    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++) {
        current_spreadsheet_state.underlying_cell_data.push([]);
        for(var j = 0;j < current_spreadsheet_state.num_columns;j ++) {
            current_spreadsheet_state.underlying_cell_data[i].push("");
            populate_single_cell(container, i, j);
        }
    }
}

function load_data() {
    if(current_spreadsheet_state.cell_data.length == 0) return;

    for(var i = 0;i < current_spreadsheet_state.cell_merge_data.length;i ++) {
        var merge = current_spreadsheet_state.cell_merge_data[i];
        merge_cells(merge.x1, merge.x2, merge.y1, merge.y2, false);
    }

    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++) {
        for(var j = 0;j < current_spreadsheet_state.num_columns;j ++) {
            var element = document.getElementById("spreadsheet-" + i + ":" + j);
            if(element != null) {
                element.value = current_spreadsheet_state.cell_data[i][j];
                element.className = current_spreadsheet_state.cell_style_classes[i][j];
            }
        }
    }
}

function umerge_cells(element) {
    var x1, x2, y1, y2;

    x1 = parseInt(element.style.gridColumnStart) - 2;
    x2 = parseInt(element.style.gridColumnEnd) - 3;
    y1 = parseInt(element.style.gridRowStart) - 2;
    y2 = parseInt(element.style.gridRowEnd) - 3;

    current_spreadsheet_state.cell_merge_data = current_spreadsheet_state.cell_merge_data.filter(function(e) { 
        var flag = e.x1 == x1 && e.x2 == x2 && e.y1 == y1 && e.y2 == y2;
        return !flag;
    });

    var contents = element.value;
    element.parentNode.removeChild(element);

    for(var i = y1;i <= y2;i ++) {
        for(var j = x1;j <= x2;j ++) {
            var element = document.createElement("input");
            element.id = "spreadsheet-" + i + ":" + j;
            element.style.gridColumnStart = j + 2;
            element.style.gridRowStart = i + 2;
            spreadsheet_container.appendChild(element);
            if(i == y1 && j == x1) {
                element.value = contents;
            }
        }
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
    element.classList.add("merged");
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
    current_spreadsheet_state.cell_style_classes = [];
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

    spreadsheet_states[current_spreadsheet_id] = current_spreadsheet_state;
}

function delete_spreadsheet() {
    var container = document.getElementById("spreadsheet-container");
    container.replaceChildren(selection_extents_element);//Leave the selection element
}

function switch_spreadsheet_state(new_index) {
    save_spreadsheet_state();
    delete_spreadsheet();
    spreadsheet_states[current_spreadsheet_id] = current_spreadsheet_state;
    current_spreadsheet_id = new_index;
    current_spreadsheet_state = spreadsheet_states[current_spreadsheet_id];
    create_spreadsheet();
}

function rerun_all() {
    for(var i = 0;i < current_spreadsheet_state.num_rows;i ++) {
        for(var j = 0;j < current_spreadsheet_state.num_columns;j ++) {
            var element = document.getElementById("spreadsheet-" + i + ":" + j);
            if(element.value != "") eval_spreadsheet_formula(element.value, { i: i, j:j });
        }
    }
}

create_spreadsheet();

document.body.addEventListener("keyup", function(event) {
    if (event.key === "Enter" && (document.activeElement.id.startsWith("spreadsheet") || document.activeElement.id.startsWith("lexicon"))) {
        document.activeElement.blur();
    }
});