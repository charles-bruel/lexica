var conversion_id = 0;
var program_test_references = [];

//This  function adds requests a sound change for use in the program test area
function add_program_test_area_conversion(program_name, word, line) {
    var id = request_conversion(program_name, word);
    program_test_references.push({ id: id, val: line})
}

//This function appends a request to send a word through a certain program to the list
//It will only be sent when push_conversions() is called
function request_conversion(program_name, word) {
    if(!Object.hasOwn(conversions_to_send, program_name)) conversions_to_send[program_name] = [];
    var id = conversion_id;
    conversions_to_send[program_name].push({ id: id, data: { Ok: word }});
    conversions_in_flight.push({ id: id, og: word, prog: program_name });
    conversion_id++;
    return id;
}

//This pushes all stored conversion, for every program
//Each program gets it's own request method
//If there is nothing to send, this function has no side effects
function push_conversions() {
    var keys = Object.keys(conversions_to_send);
    for(var i = 0;i < keys.length;i ++) {
        var array = conversions_to_send[keys[i]];
        send_sc_request(array, keys[i]);
        delete conversions_to_send[keys];
    }
}

//Call it push conversions periodically
//This calls it every 100ms, or 10x a second
setInterval(push_conversions, 100);

//This function extracts the result from a sound change result object
function get_sc_result(obj) {
    if(Object.hasOwn(obj, "Ok")) {
        return obj.Ok;
    } else {
        return JSON.stringify(obj.Err[Object.keys(obj.Err)[0]]);
    }
}

//This function takes a list of returned sound changer results and
//adds them to the cache, and routes them wherever else they need to go, if nessecary
function handle_sc_response(entries) {
    for(const entry of entries) {
        //Save everything to the cache
        var conversion = take_valid_entry_for_sc(conversions_in_flight, entry.id);
        if(conversion !== -1) {
            if(!Object.hasOwn(computation_cache, conversion.prog)) computation_cache[conversion.prog] = {};
            computation_cache[conversion.prog][conversion.og] = get_sc_result(entry.data);
        } else {
            //Should be unreachable, everything is supposed to get cached
            console.log("result somehow wasn't requested to be cached");
        }

        //TODO: Maybe come up with a better way of linking these
        var test_item = take_valid_entry_for_sc(program_test_references, entry.id);
        if(test_item !== -1) {
            var element = document.getElementById("program-manager-test-area");
            var temp = element.value.split("\n");
            temp[test_item.val] += " => " + get_sc_result(entry.data);
            element.value = temp.join("\n");
        }
    }
    mark_updated();
}

//This looks through and array of id/position pairs for the given id,
//if it finds it, it removes it from the array and returns it, returning -1 otherwise
//This helps to send sound changer results to the correct place
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

//This function converts an input which may have escaped characters (e.g. \\, \n, etc.) into one with just regular codepoints
//The output of this function may display as escaped characters in certain places, but these characters are still escaped
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

//AST functions
//TODO: Must check that params are available before evaluating
//TODO: Refactor params part of available() into ASTNode
const AST_functions = {
    run: {
        immediate: false,
        available: function(params) {
            if(!Object.hasOwn(computation_cache, params[0].evaluate())) {
                //Nothing with this program name has been cached
                return false;
            }
            if(!Object.hasOwn(computation_cache[params[0].evaluate()], params[1].evaluate())) {
                //This program name has been cached, but not this input
                return false;
            }
            //This program name has been cached and this input has been cached
            return true;
        },
        make_available: function(params) {
            for(x in conversions_in_flight) {
                //Check if this conversion has already been requested
                //If so, no need to send it again
                if(x.prog == params[0].evaluate() && x.og == params[1].evaluate()) return;
            }
            request_conversion(params[0].evaluate(), params[1].evaluate());
        },
        evaluate: function(params) {
            //Since we can assume available() is true, we can just immediately grab the valuea
            //and not worry about safety
            return computation_cache[params[0].evaluate()][params[1].evaluate()];
        }
    }
};

//AST nodes
//An AST node should contain the following functions
//available() returns true or false depending on whether the value is available RN
//            For example, a sound change run function would not be available if the correct operation is not cached
//immediate_available() returns true if the value is immediately and always computable. Literals and operations just on literals always fulfil this criteria
//                      but even sound change runs, even if cached would not. This should always return the same value for each node type.
//make_available() Runs code to attempt to make the expression available. For example, a sound change run might use this to ask for the value, which is then cached.
//                 This function is only required to behave well if available() === false. For some nodes, available() is always true, so this might not even be defined
//evaluate() evaluates the value of the expression, possibly recursively. This is only required to behave well if available() === true
//An AST node should also have the following property
//params should be an array of other ASTNodes, and represent every AST node that is used to evaluate this AST node
class ASTNode {
    constructor() {
        this.params = [];
    }
    make_available() {
        for(var i = 0;i < this.params.length;i ++) {
            this.params[i].make_available();
        }
    }
    available() {
        for(var i = 0;i < this.params.length;i ++) {
            if(!this.params[i].available()) return false;
        }
        return true;
    }
    immediate_available() {
        for(var i = 0;i < this.params.length;i ++) {
            if(!this.params[i].immediate_available()) return false;
        }
        return true;
    }
}

//This node represents a string literal
class ASTStringLiteralNode extends ASTNode {
    constructor(string) {
        super();
        this.literal = manage_escaped_characters(string);
    }
    evaluate() {
        return this.literal;
    }
    available() {
        return true;
    }
    immediate_available() {
        return true;
    }
}

//This node represents a number. It is also used as the boolean type, with 0 === false and everything being true
class ASTNumericLiteralNode extends ASTNode {
    constructor(value) {
        super();
        this.literal = value;
    }
    evaluate() {
        return this.literal;
    }
    available() {
        return true;
    }
    immediate_available() {
        return true;
    }
}

//This node represents a function, like a syscall. These are all system defined functions
class ASTFunctionNode extends ASTNode {
    constructor(function_name, params) {
        super();
        this.function_name = function_name;
        this.params = params;
    }
    evaluate() {
        if(Object.hasOwn(AST_functions, this.function_name)) {
            return AST_functions[this.function_name].evaluate(this.params);
        } else {
            return "ERROR: Unknown function \"" + this.function_name + "\"";
        }
    }
    available() {
        if(!super.available()) return false;
        if(Object.hasOwn(AST_functions, this.function_name)) {
            return AST_functions[this.function_name].available(this.params);
        } else {
            return true;
        }
    }
    immediate_available() {
        if(!super.immediate_available()) return false;
        if(Object.hasOwn(AST_functions, this.function_name)) {
            return AST_functions[this.function_name].immediate;
        } else {
            return false;
        }
    }
    make_available() {
        super.make_available();

        //There are two ways this not available
        //The first is if available() returned because of super.available(), i.e. one of the params was not available
        //Or this function could be unavailable due to function itself
        //The function cannot make itself available if it's parameters are not ready
        if(!super.available()) return;

        if(Object.hasOwn(AST_functions, this.function_name)) {
            AST_functions[this.function_name].make_available(this.params);
        }
    }
}

//This represents a unary operation, that is an operation on a single operand
class ASTUnaryNode extends ASTNode {
    constructor(operand) {
        super();
        this.params = [operand];
    }
}

//This represents a binary operation, that is an operation on two operands
class ASTBinaryNode extends ASTNode {
    constructor(operand_left, operand_right) {
        super();
        this.params = [operand_left, operand_right];
    }
}

//This function takes a value of unknown type and returns the correct AST literal node type
//This function assumes that the type is already correct; it will not attempt to parse a float from a string, for example
//If you put in "1.23", you get a string literal containing "1.23", not a numeric literal with 1.23
function create_AST_literal(input) {
    if(typeof input == "string") {
        return new ASTStringLiteralNode(input);
    }
    if(typeof input == "number") {
        return new ASTNumericLiteralNode(input);
    }
    return "ERROR: Unknown type"
}

//This is an intermediate representation of an AST operator, between being parsed and having the context to have other nodes assigned to it
//An AST operator should have a get_node(operand, ...) function which takes the appropiate number of operands and returns an ASTNode that represents the operation
class ASTOperator {}

function create_AST_unary_node(operand, fn) {
    var temp = new ASTUnaryNode(operand);
    temp.evaluate = fn;
    return temp;
}

function create_AST_binary_node(operand_left, operand_right, fn) {
    var temp = new ASTBinaryNode(operand_left, operand_right);
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
                return create_AST_unary_node(operand, function() { return -this.params[0].evaluate(); });
            case "!":
                return create_AST_unary_node(operand, function() { return this.params[0].evaluate() == 0 ? 1 : 0 });
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
                return create_AST_binary_node(operand_left, operand_right, function() { return this.params[0].evaluate() + this.params[1].evaluate(); });
            case "-":
                return create_AST_binary_node(operand_left, operand_right, function() { return this.params[0].evaluate() - this.params[1].evaluate(); });
        }
        return "TODO";
    }
}

//This class represents an AST node that is waiting on something to complete. It contains the partially compiled AST tree and the a callback
//which will be called when the computation finishes so that the value can be assigned.
class ASTJob {
    constructor(ast, callback) {
        this.ast = ast;
        this.callback = callback;
    }

    ready_now() {
        return this.ast.available();
    }

    //This assumes ready_now() returned true
    evaluate() {
        return this.ast.evaluate();
    }

    finish(final_value) {
        this.callback(final_value);
    }
}