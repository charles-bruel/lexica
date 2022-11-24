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

function manage_escaped_characters(input) {
    var result = "";
    for(var i = 0;i < input.length;i ++) {
        //Detect the beggining of escapes
        if(input == "\\") {
            if(i == input.length - 1) {
                //This *should* be unreachable
                //For their to be a backslash here, the last character would need to be a backslash, escaping the end quotes.
                //If the backslash is escaped by a backslash before it, this backslash would not be processsed
            } else {
                if(input[i + 1] == "n") {
                    result += "\n";//Add \n support
                } else {
                    //We don't support most of the javascript special escapes, and all the remaining ones (\", \', \\) just copy the character.
                    //It should be UB to escape anything else, but just keeping it as-is should be simpler.
                    result += input[i+1];
                }
            }
        } else {
            result += input[i];
        }
    }
    return result;
}

//AST nodes
class ASTNode {}

class ASTStringLiteralNode extends ASTNode {
    constructor(string) {
        super();
        this.literal = manage_escaped_characters(string);
    }
    evaluate() {
        return this.literal;
    }
    avaliable() {
        return true;
    }
}

class ASTNumericLiteralNode extends ASTNode {
    constructor(value) {
        super();
        this.literal = value;
    }
    evaluate() {
        return this.literal;
    }
    avaliable() {
        return true;
    }
}

class ASTFunctionNode extends ASTNode {
    constructor(function_name, params) {
        super();
        this.function_name = function_name;
        this.params = params;
    }
    evaluate() {
        return "TODO";
    }
    avaliable() {
        return "TODO";
    }
}

class ASTUnaryNode extends ASTNode {
    constructor(operand) {
        super();
        this.operand = operand;
    }
    avaliable() {
        return this.operand.avaliable();
    }
}

class ASTBinaryNode extends ASTNode {
    constructor(operand_left, operand_right) {
        super();
        this.operand_left = operand_left;
        this.operand_right = operand_right;
    }
    avaliable() {
        return this.operand_left.avaliable() && this.operand_right.avaliable();
    }
}

class ASTOperator {}

function create_AST_unary_node(fn) {
    var temp = Object.create(ASTUnaryNode);
    temp.evaluate = fn;
    return temp;
}

function create_AST_binary_node(fn) {
    var temp = Object.create(ASTBinaryNode);
    temp.evaluate = fn;
    return temp;
}

class ASTUnaryOperator extends ASTOperator {
    //TODO: Find away to use "-" as both a unary and binary operator
    static operators = ["~", "!"];

    //operator must be one of the above strings
    constructor(operator) {
        super();
        this.operator = operator;
    }

    get_node(operand) {
        switch(this.operator) {
            case "~":
                return create_AST_unary_node(function() { return -operand.evaluate(); });
            case "!":
                return create_AST_unary_node(function() { return operand.evaluate() == 0 ? 1 : 0 });
        }
        return "TODO";
    }
}

class ASTBinaryOperator extends ASTOperator {
    static operators = ["+", "-", "*", "/", "==", "!=", ">", "<", ">=", "<="];

    //operator must be one of the above strings
    constructor(operator) {
        super();
        this.operator = operator;
    }

    get_node(operand_left, operand_right) {
        switch(this.operator) {
            case "+":
                return create_AST_binary_node(function() { return operand_left.evaluate() + operand_right.evaluate(); });
            case "-":
                return create_AST_binary_node(function() { return operand_left.evaluate() - operand_right.evaluate(); });
        }
        return "TODO";
    }
}