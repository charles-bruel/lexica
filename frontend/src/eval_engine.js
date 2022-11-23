var active_conversions = {};
var conversion_id = 0;
var conversions_to_send = {};
var spreadsheet_references = [];
var program_test_references = [];

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