var active_conversions = {};
var conversion_id = 0;
var conversions_to_send = {};
var spreadsheet_references = [];

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
    if(!Object.hasOwn(conversions_to_send, params[0])) conversions_to_send[params[0]] = [];
    conversions_to_send[params[0]].push({ id: conversion_id, data: { Ok: params[1] }});
    spreadsheet_references.push({ id: conversion_id++, val: pos})
    return "AWAITING RESULT";
}

function eval_value(input, pos) {
    if(!input.startsWith("=")) {
        return input;
    }
    input = input.substring(1);
    // TODO: Create fully recursive evaluator
    if(input.startsWith("run")) {
        input = handle_run(input, pos);
    }
    push_conversions();
    return input;
}

function push_conversions() {
    var keys = Object.keys(conversions_to_send);
    for(var i = 0;i < keys.length;i ++) {
        var array = conversions_to_send[keys[i]];
        send_sc_request(array, keys[i]);
        delete conversions_to_send[keys];
    }
}

function handle_sc_response(entries) {
    console.log(entries);
    for(const entry of entries) {
        if(spreadsheet_references[entry.id] !== null) {
            var ref = spreadsheet_references[entry.id];
            var element = document.getElementById("spreadsheet-" + ref.val.i + ":" + ref.val.j);
            element.value = entry.data.Ok;
        }
    }
}