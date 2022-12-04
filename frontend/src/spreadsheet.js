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
var spreadsheet_edit_cell_text = false;

var spreadsheet_cell_editor = document.getElementById("spreadsheet-cell-editor");

spreadsheet_cell_editor.addEventListener("blur", function(e) { handle_spreadsheet_cell_editor_blur(e, spreadsheet_cell_editor) })
spreadsheet_cell_editor.addEventListener("input", function() { handle_spreadsheet_cell_input(spreadsheet_cell_editor); });

const spreadsheet_cell_id_regex = /spreadsheet-[0-9]+:[0-9]+/;

var selection_base_pos;
var selection_extent_pos;
var selection_base_element;
var selection_mode_active;

selection_extents_element_a = document.getElementById("spreadsheet-selection-extents-a");
selection_extents_element_b = document.getElementById("spreadsheet-selection-extents-b");
selection_extents_element_c = document.getElementById("spreadsheet-selection-extents-c");

function set_root_variable(variable_name, value) {
    var r = document.querySelector(':root');
    r.style.setProperty(variable_name, value);
}
set_spreadsheet_text_mode(false);

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

function handle_spreadsheet_cell_editor_blur(e, element) {
    if(e.relatedTarget == null || !spreadsheet_cell_id_regex.test(e.relatedTarget.id)) {
        blur_spreadsheet_selection();
        var i = selection_base_pos.i;
        var j = selection_base_pos.j;
        current_spreadsheet_state.underlying_cell_data[i][j] = element.value;
        selection_base_element.value = eval_spreadsheet_formula(current_spreadsheet_state.underlying_cell_data[i][j], { i: i, j:j });  
    }
    spreadsheet_cell_editor.value = "";
}

function blur_spreadsheet_selection() {
    last_focused_cell = null;
    set_spreadsheet_text_mode(false);

    selection_extents_element_a.style.display = "none";
    selection_extents_element_b.style.display = "none";
    selection_extents_element_c.style.display = "none";
}

function handle_spreadsheet_cell_blur(e, element, i, j) {
    if(spreadsheet_edit_cell_text) {
        current_spreadsheet_state.underlying_cell_data[i][j] = element.value;
        element.value = eval_spreadsheet_formula(current_spreadsheet_state.underlying_cell_data[i][j], { i: i, j:j });
    }

    if(e.relatedTarget != spreadsheet_cell_editor) {
        blur_spreadsheet_selection();
        if(e.relatedTarget == null || !spreadsheet_cell_id_regex.test(e.relatedTarget.id)) {
            spreadsheet_cell_editor.value = "";
        }
    }
}

function handle_spreadsheet_cell_focus(element, i, j) {
    spreadsheet_cell_editor.value = element.value;

    if(selection_mode_active) { 
        show_spreadsheet_selection_extents();
        return;
    }
    selection_base_pos = { i: i, j: j };
    selection_extent_pos = { i: i, j: j };
    selection_base_element = element;
    show_spreadsheet_selection_extents();
}

function populate_single_cell(container, i, j) {
    var element = document.createElement("input");
    element.id = "spreadsheet-" + i + ":" + j;
    element.className = "spreadsheet-cell";
    element.style.gridColumnStart = j + 2;
    element.style.gridRowStart = i + 2;
    container.appendChild(element);
    element.addEventListener("blur", function(e) { handle_spreadsheet_cell_blur(e, element, i, j); });
    element.addEventListener("focus", function() { handle_spreadsheet_cell_focus(element, i, j); });
    element.addEventListener("input", function() { handle_spreadsheet_cell_input(element); });
    element.addEventListener("dblclick", e => {
        set_spreadsheet_text_mode(true);
    })
}

function set_spreadsheet_text_mode(val) {
    spreadsheet_edit_cell_text = val;
    set_root_variable("--spreadsheet-textedit", val ? "white" : "transparent");
    if(val) {
        var i = selection_base_pos.i;
        var j = selection_base_pos.j;    
        var new_value = current_spreadsheet_state.underlying_cell_data[i][j];

        selection_base_element.value = new_value;
        spreadsheet_cell_editor.value = new_value;
    } else {

    }
}

//This function is generic and handles both cell 
//editor bar and editing in the cell itself
function handle_spreadsheet_cell_input(element) {
    var cached_value = element.value;
    if(!spreadsheet_edit_cell_text) set_spreadsheet_text_mode(true);

    var new_value = element.value;
    if(new_value === "") {
        new_value = cached_value;
    }

    var i = selection_base_pos.i;
    var j = selection_base_pos.j;
    current_spreadsheet_state.underlying_cell_data[i][j] = new_value;
    selection_base_element.value = new_value;
    spreadsheet_cell_editor.value = new_value;
}

function show_spreadsheet_selection_extents() {
    //Border
    selection_extents_element_c.style.display = "block";

    var cmini = Math.min(selection_base_pos.i, selection_extent_pos.i);
    var cmaxi = Math.max(selection_base_pos.i, selection_extent_pos.i);
    var cminj = Math.min(selection_base_pos.j, selection_extent_pos.j);
    var cmaxj = Math.max(selection_base_pos.j, selection_extent_pos.j);
    selection_extents_element_c.style.gridRowStart = cmini + 2;
    selection_extents_element_c.style.gridRowEnd = cmaxi + 3;
    selection_extents_element_c.style.gridColumnStart = cminj + 2;
    selection_extents_element_c.style.gridColumnEnd = cmaxj + 3;


    //Selection highlight
    //Special cases
    if(selection_base_pos.i == selection_extent_pos.i && selection_base_pos.j == selection_extent_pos.j) {
        selection_extents_element_a.style.display = "none";
        selection_extents_element_b.style.display = "none";
        return;
    }

    if(selection_base_pos.i == selection_extent_pos.i) {
        selection_extents_element_b.style.display = "none";

        var aminj;
        var amaxj;
        if(selection_base_pos.j < selection_extent_pos.j) {
            aminj = selection_base_pos.j + 1;
            amaxj = selection_extent_pos.j;
        } else {
            aminj = selection_extent_pos.j;
            amaxj = selection_base_pos.j - 1;
        }
        var ai = selection_base_pos.i;

        selection_extents_element_a.style.gridRowStart = ai + 2;
        selection_extents_element_a.style.gridRowEnd = ai + 3;
        selection_extents_element_a.style.gridColumnStart = aminj + 2;
        selection_extents_element_a.style.gridColumnEnd = amaxj + 3;
        return;
    }


    //A will take the column of the main selected element, excluding that element
    //B will take the block of everything else, excluding A's column
    selection_extents_element_a.style.display = "block";

    var amini;
    var amaxi;
    if(selection_base_pos.i < selection_extent_pos.i) {
        amini = selection_base_pos.i + 1;
        amaxi = selection_extent_pos.i;
    } else {
        amini = selection_extent_pos.i;
        amaxi = selection_base_pos.i - 1;
    }
    var aj = selection_base_pos.j;

    selection_extents_element_a.style.gridRowStart = amini + 2;
    selection_extents_element_a.style.gridRowEnd = amaxi + 3;
    selection_extents_element_a.style.gridColumnStart = aj + 2;
    selection_extents_element_a.style.gridColumnEnd = aj + 3;

    //Deferred special case, a already does what we want
    if(selection_base_pos.j == selection_extent_pos.j) {
        selection_extents_element_b.style.display = "none";
        return;
    }

    selection_extents_element_b.style.display = "block";

    var bmini = Math.min(selection_base_pos.i, selection_extent_pos.i);
    var bmaxi = Math.max(selection_base_pos.i, selection_extent_pos.i);
    var bminj;
    var bmaxj;
    if(selection_base_pos.j < selection_extent_pos.j) {
        bminj = selection_base_pos.j + 1;
        bmaxj = selection_extent_pos.j;
    } else {
        bminj = selection_extent_pos.j;
        bmaxj = selection_base_pos.j - 1;
    }

    selection_extents_element_b.style.gridRowStart = bmini + 2;
    selection_extents_element_b.style.gridRowEnd = bmaxi + 3;
    selection_extents_element_b.style.gridColumnStart = bminj + 2;
    selection_extents_element_b.style.gridColumnEnd = bmaxj + 3;

}

document.addEventListener('mousemove', e => {
    if(!selection_mode_active) return;
    var element = document.elementFromPoint(e.clientX, e.clientY);
    if(!element.classList.contains("spreadsheet-cell")) return;
    var id = element.id;
    id = id.substring(12);
    var nums = id.split(":");
    var i = parseInt(nums[0]);
    var j = parseInt(nums[1]);
    selection_extent_pos = { i: i, j: j };
    show_spreadsheet_selection_extents();
});

document.addEventListener('mousedown', e => {
    if(selection_mode_active) return;
    var element = document.elementFromPoint(e.clientX, e.clientY);
    if(!element.classList.contains("spreadsheet-cell")) return;
    var id = element.id;
    id = id.substring(12);
    var nums = id.split(":");
    var i = parseInt(nums[0]);
    var j = parseInt(nums[1]);
    selection_base_pos = { i: i, j: j };
    selection_extent_pos = { i: i, j: j };
    selection_base_element = element;
    selection_mode_active = true;
    show_spreadsheet_selection_extents();
});

document.addEventListener('mouseup', e => {
    selection_mode_active = false;
    var element = document.elementFromPoint(e.clientX, e.clientY);
});


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
                element.value = eval_spreadsheet_formula(current_spreadsheet_state.underlying_cell_data[i][j], { i: i, j:j });
                element.className = current_spreadsheet_state.cell_style_classes[i][j];
                if(!element.classList.contains("spreadsheet-cell")) {
                    element.classList.add("spreadsheet-cell");
                }
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
    var new_class_name = element.className.replace("merged", "").trim();

    for(var i = y1;i <= y2;i ++) {
        for(var j = x1;j <= x2;j ++) {
            var new_element = document.createElement("input");
            new_element.id = "spreadsheet-" + i + ":" + j;
            new_element.style.gridColumnStart = j + 2;
            new_element.style.gridRowStart = i + 2;
            new_element.className = new_class_name;
            spreadsheet_container.appendChild(new_element);
            if(i == y1 && j == x1) {
                new_element.value = contents;
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
    element.classList.add("spreadsheet-cell");
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
                var new_class_name = element.className.replace("spreadsheet-cell", "").trim();
                current_spreadsheet_state.cell_style_classes[i].push(new_class_name);
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
    container.replaceChildren(selection_extents_element_a, selection_extents_element_b, selection_extents_element_c);//Leave the selection element
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

function get_elements_for_modification() {
    var element = document.activeElement;
    if (element == null) return [];
    if (element.id == "") return [];
    if (element.id.startsWith("lexicon")) {
        return [element];
    } else if (element.id.startsWith("spreadsheet")) {
        var mini = Math.min(selection_base_pos.i, selection_extent_pos.i);
        var maxi = Math.max(selection_base_pos.i, selection_extent_pos.i);
        var minj = Math.min(selection_base_pos.j, selection_extent_pos.j);
        var maxj = Math.max(selection_base_pos.j, selection_extent_pos.j);
        var result = [];

        for(var i = mini;i <= maxi;i ++) {
            for(var j = minj;j <= maxj;j ++) {
                var temp = document.getElementById("spreadsheet-" + i + ":" + j);
                if(temp != null) result.push(temp);
            }
        }
        return result;
    } else {
        return [];
    }
}

document.body.addEventListener("keyup", function(event) {
    if (event.key === "Enter" && (document.activeElement.id.startsWith("spreadsheet") || document.activeElement.id.startsWith("lexicon"))) {
        if(spreadsheet_cell_id_regex.test(document.activeElement.id)) {
            var id = document.activeElement.id;
            id = id.substring(12);
            var nums = id.split(":");
            var i = parseInt(nums[0]);
            var j = parseInt(nums[1]);
            if(event.shiftKey) {
                i--;
            } else {
                i++;
            }

            var try_element = document.getElementById("spreadsheet-" + i + ":" + j);
            if(try_element != null) {
                try_element.focus();
            }
            return;
        }
        document.activeElement.blur();
    }

    if(document.activeElement.id.startsWith("spreadsheet") && spreadsheet_cell_id_regex.test(document.activeElement.id)) {
        var id = document.activeElement.id;
        id = id.substring(12);
        var nums = id.split(":");
        var i = parseInt(nums[0]);
        var j = parseInt(nums[1]);

        var flag = false;
        var di = 0;
        var dj = 0;
        if(event.key === "ArrowLeft" && !spreadsheet_edit_cell_text) {
            dj = -1;
            flag = true;
        }
        if(event.key === "ArrowRight" && !spreadsheet_edit_cell_text) {
            dj = +1;
            flag = true;
        }
        if(event.key === "ArrowUp") {
            di = -1;
            flag = true;
        }
        if(event.key === "ArrowDown") {
            di = +1;
            flag = true;
        }

        if(flag) {
            if(event.shiftKey) {
                selection_extent_pos.i += di;
                selection_extent_pos.j += dj;
                show_spreadsheet_selection_extents();
            } else {
                var id = document.activeElement.id;
                id = id.substring(12);
                var nums = id.split(":");
                var i = parseInt(nums[0]) + di;
                var j = parseInt(nums[1]) + dj;
                var try_element = document.getElementById("spreadsheet-" + i + ":" + j);
                if(try_element != null) {
                    try_element.focus();
                }
            }
            return;
        }

        if(event.key === "Delete" && !spreadsheet_edit_cell_text) {
            var elements = get_elements_for_modification();
            for(const element of elements) {
                element.value = "";

                //Highly inefficient however it probably doesn't matter
                //Also yes I know I'm using var wrong. I really should switch to let
                var id = document.activeElement.id;
                id = id.substring(12);
                var nums = id.split(":");
                var i = parseInt(nums[0]);
                var j = parseInt(nums[1]);        
                current_spreadsheet_state.underlying_cell_data[i][j] = "";        
                spreadsheet_cell_editor.value = "";
            }
            return;
        }

        if(event.key === "F2") {
            selection_base_pos.i = i;
            selection_base_pos.j = j;
            selection_extent_pos.i = i;
            selection_extent_pos.j = j;
            selection_base_element = document.activeElement;
            set_spreadsheet_text_mode(true);
            return;
        }

        if(event.key === "Escape") {
            set_spreadsheet_text_mode(false);
            var i = selection_base_pos.i;
            var j = selection_base_pos.j;
            var element = document.getElementById("spreadsheet-" + i + ":" + j);
            current_spreadsheet_state.underlying_cell_data[i][j] = element.value;
            element.value = eval_spreadsheet_formula(current_spreadsheet_state.underlying_cell_data[i][j], { i: i, j:j });
            return; 
        }
    }
});