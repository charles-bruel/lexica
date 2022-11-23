function get_top_level_parens_content(input) {
    var start = input.indexOf("(");
    var end = input.lastIndexOf(")");
    if(start == -1 || end == -1) return input;
    return input.substring(start + 1, end);
}

const splitter_re = / (?![^(]*\))(?![^\[]*\])/;

function handle_run(input, pos) {
    input = get_top_level_parens_content(input);
    var params = input.split(splitter_re);
    var program_name = params[0];
    var word = params[1];
    if(!Object.hasOwn(conversions_to_send, program_name)) conversions_to_send[program_name] = [];
    conversions_to_send[program_name].push({ id: conversion_id, data: { Ok: word }});
    spreadsheet_references.push({ id: conversion_id++, val: pos});
    return "AWAITING RESULT";
}

function add_program_test_area_conversion(program_name, word, line) {
    if(!Object.hasOwn(conversions_to_send, program_name)) conversions_to_send[program_name] = [];
    conversions_to_send[program_name].push({ id: conversion_id, data: { Ok: word }});
    program_test_references.push({ id: conversion_id++, val: line})
}

function push_conversions() {
    var keys = Object.keys(conversions_to_send);
    for(var i = 0;i < keys.length;i ++) {
        var array = conversions_to_send[keys[i]];
        send_sc_request(array, keys[i]);
        delete conversions_to_send[keys];
    }
}

function take_valid_entry_for_sc(array, id) {
    for(var i = 0;i < array.length;i ++) {
        if(array[i].id === id) {
            var temp = array[i];
            array.splice(i, 1);
            return temp;
        }
    }
    return -1;
}

function get_sc_result(obj) {
    if(Object.hasOwn(obj, "Ok")) {
        return obj.Ok;
    } else {
        return JSON.stringify(obj.Err[Object.keys(obj.Err)[0]]);
    }
}

function handle_sc_response(entries) {
    console.log(entries);
    for(const entry of entries) {
        var spreadsheet_item = take_valid_entry_for_sc(spreadsheet_references, entry.id);
        if(spreadsheet_item !== -1) {
            var element = document.getElementById("spreadsheet-" + spreadsheet_item.val.i + ":" + spreadsheet_item.val.j);
            element.value = get_sc_result(entry.data);
        }
        var test_item = take_valid_entry_for_sc(program_test_references, entry.id);
        if(test_item !== -1) {
            var element = document.getElementById("program-manager-test-area");
            var temp = element.value.split("\n");
            temp[test_item.val] += " => " + get_sc_result(entry.data);
            element.value = temp.join("\n");
        }
    }
}