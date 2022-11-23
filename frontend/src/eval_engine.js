var active_conversions = {};
var conversion_id = 0;
var conversions_to_send = {};
var spreadsheet_references = [];
var program_test_references = [];

//This AST ultimately evaluates to a single node which can be evaluated or placed into a parent AST
function create_ast(input) {
    //Process:
    //Divide context into values and operators
    //e.g. [<value>, <+>, <value>]
    //Values are space seperated. Quotes and curly braces act as groupers
    //Functions are interpreated similarly
    for(var i = 0;i < input.length;i ++) {
        var nodes = [];
        //Test for grouping
        if(input[i] == "{") {
            //Find the end of the grouping section.
            let depth = 1;//One curly brace in
            for(var j = i + 1;j < input.length;j ++) {
                if(input[j] == "{") {
                    depth++;
                }
                if(input[j] == "}") {
                    depth--;
                }
                if(depth == 0) {
                    //Add the AST as a value node represented by the contents of this group
                    //Begin is inclusive, so ignore the opening curly brace
                    //End is exclusive, so no need to modify
                    var inner_contents = input.substring(i + 1, j);
                    nodes.push(create_ast(inner_contents));
                    //Rebase i to after the group. i is now on the end curly brace, when i++ is called, it will be the
                    //character after the group ends
                    i = j;
                    break;
                }
            }
        }
        if(input[i] == "\"") {
            for(var j = i + 1;j < input.length;j ++) {
                if(input[i] == "\"") {
                    //Check if it has been escaped
                    if(input[j-1] != "\\") {
                        //Actual end of quote
                        //Add the contents of the quotes as a literal
                        //Begin is inclusive, so ignore the opening curly brace
                        //End is exclusive, so no need to modify
                        var inner_contents = input.substring(i + 1, j);
                        nodes.push(create_string_literal_ast_node(inner_contents));
                    }
                }
            }
        }
    }
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