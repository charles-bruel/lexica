const color_options = ["Brown", "BlueViolet", "CadetBlue", "Chocolate", "CornflowerBlue", "DarkGoldenRod", "DarkOliveGreen", "DarkSalmon", "LightSlateGray", "Gray", "MidnightBlue", "RebeccaPurple"];

var bottom_bar_state = {
    names: ["Tab 1"],
    elems: [],
    colors: [color_options[Math.floor(Math.random()*color_options.length)]],
    selected_index: 0,
    parent_elem: null,
    tab_counter: 1,
};

//This handles selecting a tab
function handle_bottom_bar_button(index) {
    var startTime = performance.now();

    bottom_bar_state.elems[bottom_bar_state.selected_index].classList.remove("selected");
    bottom_bar_state.selected_index = index;
    bottom_bar_state.elems[bottom_bar_state.selected_index].classList.add("selected");

    set_root_variable("--color", bottom_bar_state.colors[bottom_bar_state.selected_index]);

    switch_spreadsheet_state(index);
    // switch_lexicon_state(index); // TODO: Default stops associated with tabs

    document.getElementById("top-bar-tab-name").textContent = bottom_bar_state.names[bottom_bar_state.selected_index];

    var endTime = performance.now();
    console.log(`Call to handle_bottom_bar_button took ${endTime - startTime} milliseconds`);
}

//This handles finishing renaming a tab
function handle_end_bottom_bar_rename(index, input_element) {
    bottom_bar_state.elems[index].style.display = "";
    var new_name = input_element.value;
    input_element.remove(input_element);
    bottom_bar_state.elems[index].textContent = new_name;
    bottom_bar_state.names[index] = new_name;

    document.getElementById("top-bar-tab-name").textContent = bottom_bar_state.names[index];
}

//This handles starting to rename a tab
function handle_bottom_bar_rename(index) {
    bottom_bar_state.elems[index].style.display = "none";
    var element = document.createElement("input");
    bottom_bar_state.elems[index].parentElement.insertBefore(element, bottom_bar_state.elems[index]);
    element.defaultValue = bottom_bar_state.names[index];
    element.focus();
    element.select();
    element.addEventListener("focusout", function() { handle_end_bottom_bar_rename(index, element); })
}

//This handles creating a new button
function generate_bottom_bar_button(index) {
    var element = document.createElement("button");
    element.textContent = bottom_bar_state.names[index];
    element.addEventListener("click", function() { handle_bottom_bar_button(index);})
    element.addEventListener("dblclick", function() { handle_bottom_bar_rename(index);})
    bottom_bar_state.elems[index] = element;
    element.style.borderColor = bottom_bar_state.colors[index];
    return element;
}

//This handles adding a new tab
function add_bottom_bar_button(name) {
    bottom_bar_state.names.push(name);
    parent_elem = document.getElementById("bottom-bar-buttons");
    var element = generate_bottom_bar_button(bottom_bar_state.names.length - 1);
    parent_elem.appendChild(element);
    bottom_bar_state.colors.push(color_options[Math.floor(Math.random()*color_options.length)]);
    spreadsheet_states.push(new_spreadsheet_state());
    handle_bottom_bar_button(spreadsheet_states.length - 1);
}

function create_bottom_bar() {
    parent_elem = document.getElementById("bottom-bar-buttons");

    for(var i = 0;i < bottom_bar_state.names.length;i ++) {
        var element = generate_bottom_bar_button(i);
        parent_elem.appendChild(element);
    }

    new_tab_button = document.getElementById("new-tab");
    new_tab_button.addEventListener("click", function() { add_bottom_bar_button("New tab " + bottom_bar_state.tab_counter++) } )
}

function delete_bottom_bar() {
    var container = document.getElementById("bottom-bar-buttons");
    container.replaceChildren();
}

create_bottom_bar();
handle_bottom_bar_button(bottom_bar_state.selected_index);
set_root_variable("--select-color", "blue");