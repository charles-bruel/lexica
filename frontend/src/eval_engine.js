//This stores every word transformation for the current iteration of the program
//It's stored as a two level dictionary with the first level property names being the program name
//and the second level being the input word
//e.g. { foo: { bar: "baz" }} means that when running foo, bar => baz
var computation_cache = {};

//This stores all in progress computations that are waiting on something
var computation_queue = [];

//This stores every requested sound change, so that they can be batched
//This is an object with the program name as the property name, and that property contains an array of everything that needs to be sent
//This contains requests from both the program menu test area and from the evaluator
var conversions_to_send = {};

//This stores a list of all the conversion currently in progress, their id, and the program name associated with them
//This is used to link the results to their source and program for cache storing. Every sound change request goes through this system,
//even ones not through the evaluator
var conversions_in_flight = [];

//This will evaluate into an array of ASTNodes, with each ASTNode coming from a single section of comma seperated input
//Note this is context aware; it only splits on the highest level
//If the output is a single node, the output of this function can be evaluated or placed into another AST tree
//The function supports an array of outputs for handling function parameters
function create_AST(input, expect_single=false) {
    //Process:
    //Divide context into values and operators
    //e.g. [<value>, <+>, <value>]
    //Values are space seperated. Quotes and curly braces act as groupers
    //Functions are interpreted similarly

    //It's worth noting that when finding groups, only the exact type of character is matched.
    //For example, it would think in the string {(foo}) the contents of the braces is (foo, despite that being mismatched
    //This is fine because the error will be caught in the next layer

    //Nodes will contain a running tally of the current nodes. When a comma or EOS is encountered, convert the current set of nodes into an ASTNode,
    //append it to the result, and move on
    //nodes will be filled with a combination of ASTNodes and Operators.
    var nodes = [];
    var results = [];
    var running_string = "";

    input += ",";//Little hack to ensure the compaction process after every section, in the same place/way

    for(var i = 0;i < input.length;i ++) {
        //We immediately check to see if the section should end
        //The section should end if the current character is an operator or if the running_string is an operator (among other conditions)
        //If the running_string is an operator, we must deal with that immediately before dealing with grouping or anything else
        if(check_end_of_symbol(running_string, input[i])) {
            if(running_string.trim() == "") {
                //Couple of options here. Either it just finished with a group/quote, in which case whitespace is normal or there are multiple whitespace characters
                //Either way, the correct thing is to do nothing
            } else {
                //Whatever has been accumulating is finished, so figure out what it is and add it
                nodes.push(get_AST_node(running_string.trim()));
                running_string = "";
                //We handled the end of the previous running string intead of the current character, so we have to go back
                //This will never loop because the above condition of an empty running string will cause a fall through
                i--;
            }
        } else
        //Test for grouping
        if(input[i] == "{") {
            //Find the end of the grouping section
            let flag = false;
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
                    nodes.push(create_AST(inner_contents, true));
                    //Rebase i to after the group. i is now on the end curly brace, when i++ is called, it will be the
                    //character after the group ends
                    i = j;
                    flag = true;
                    break;
                }
            }
            if(!flag) return "ERROR: group parameter section never finishes";
        } else
        //Test for function parameters
        //In this case the contents of running_string is the function name
        if(input[i] == "(") {
            //Find the end of the parentheses section
            let flag = false;
            let depth = 1;//One paren in
            for(var j = i + 1;j < input.length;j ++) {
                if(input[j] == "(") {
                    depth++;
                }
                if(input[j] == ")") {
                    depth--;
                }
                if(depth == 0) {
                    //Get the parameter array as an AST[]
                    //Begin is inclusive, so ignore the opening curly brace
                    //End is exclusive, so no need to modify
                    var inner_contents = input.substring(i + 1, j);
                    var params = create_AST(inner_contents);
                    nodes.push(new ASTFunctionNode(running_string, params));
                    running_string = "";
                    //Rebase i to after the group. i is now on the end curly brace, when i++ is called, it will be the
                    //character after the group ends
                    i = j;
                    flag = true;
                    break;
                }
            }
            if(!flag) return "ERROR: Function parameter section never finishes";
        } else
        //Test for string literals
        if(input[i] == "\"" || input[i] == "\'") {
            var to_match = input[i];
            for(var j = i + 1;j < input.length;j ++) {
                if(input[j] == to_match) {
                    //Check if it has been escaped
                    //TODO: This does fail if the string ends with ...\\", it will incorrect treat the quote as escaped even though the backslash is escaped
                    if(input[j-1] != "\\") {
                        //Actual end of quote
                        //Add the contents of the quotes as a literal
                        //Begin is inclusive, so ignore the opening quote
                        //End is exclusive, so no need to modify
                        var inner_contents = input.substring(i + 1, j);
                        nodes.push(new ASTStringLiteralNode(inner_contents));
                        //Rebase i to after the group. i is now on the end quote, when i++ is called, it will be the
                        //character after the group ends
                        i = j;
                        break;
                    }
                }
            }
        } else
        //Finally do the accumulation
        if(input[i] != ",") {
            //No whitespace or anything else; continue as normal
            //This is placed here so the compaction section can go in it's natural place at the end
            running_string += input[i];
        } else {
            //Implicit if(input[i] == ",")
            
            //nodes is now filled with ASTNodes and Operators
            //This section will then compact Operators and ASTNodes into ASTNode(s)
            //Parentheses will already be taken care of by this point
            //All literals/immediate values should already be in ASTNodes
            
            //Process:
            //First apply unary operators
            //Then, going left to right, apply binary operators
            //nodes will be used as working space
            //Apply unary operators
            for(var j = 0;j < nodes.length;j ++) {
                if(nodes[j] instanceof ASTUnaryOperator) {
                    if(j == nodes.length - 1) {
                        //The unary operator is the final node, therefore it has no operand
                        return "ERROR: Hanging unary operator"
                    }
                    nodes[j] = nodes[j].get_node(nodes[j + 1]);//Replace the operator with a node with the operand reference
                    nodes.splice(j + 1, 1);//Remove the j + 1th item from the array
                    //j will automatically be place on the next item, no need to rebase
                }
            }

            //Apply binary operators
            for(var j = 0;j < nodes.length;j ++) {
                if(nodes[j] instanceof ASTBinaryOperator) {
                    if(j == 0 || j == nodes.length - 1) {
                        //The binary operator is the final node or it is the first node, therefore it missing an operand
                        return "ERROR: Hanging binary operator"
                    }
                    nodes[j - 1] = nodes[j].get_node(nodes[j - 1], nodes[j + 1]);//Replace the operator with a node with the operand reference
                    nodes.splice(j, 2);//Remove the jth and j + 1th item from the array
                    //Removes the original operator and the following value, leaving the node where the first operand was
                    //j is will have now skipped an item, so go 1 back
                    j--;
                }
            }

            if(nodes.length == 0) {
                return "ERROR: Blank input";
            }
            if(nodes.length > 1) {
                console.log(nodes);
                return "ERROR: When collapsing, was left with multiple sections";
            }

            results.push(nodes[0]);//Save result
            nodes = [];//Reset nodes for next parameter
        }
    }

    //If we expect a single result, return a single result for convience
    //Also allows some error checking
    //TODO: Maybe find a better error checking approach
    //I really miss rust results...
    if(expect_single) {
        if(results.length == 0) {
            return "ERROR: Empty group"
        } else if(results.length == 1) {
            return results[0];
        } else {
            return "ERROR: Expected single node, got " + results.length;
        }
    }

    return results;
}

//This function checks if the symbol should end now
//This detects stuff like "5+4", with just checking whitespace, the system would falsely interpret that as a single literal
//Not having to do an ugly hack like this is the advantage of using a tokenizer before creating an AST
//However, given the simplicity of the language, this is fine but is a potential point for improvement
function check_end_of_symbol(running_string, to_add) {
    if(/\s/.test(to_add)) return true;
    if(to_add == "," && running_string != "") return true;
    for(const x of ASTUnaryOperator.operators) {
        if(x === to_add && running_string != "") return true;
        if(x === running_string) return true;
    }
    for(const x of ASTBinaryOperator.operators) {
        if(x === to_add && running_string != "") return true;
        if(x === running_string) return true;
    }
    return false;
}

//This turns a given input into an AST node
//The name is actually slightly incorrect, as it might return an operator or an AST node
//Grouping, functions, and string literals are assumed to have been dealt with; this just deals with operators and literals
function get_AST_node(input) {
    //Process:
    //First check if it is an operator
    //Then try to convert it into a literal
    
    for(const x of ASTUnaryOperator.operators) {
        if(x === input) {
            return new ASTUnaryOperator(x);
        }
    }

    for(const x of ASTBinaryOperator.operators) {
        if(x === input) {
            return new ASTBinaryOperator(x);
        }
    }

    //TODO: Implement other literal types
    //Perhaps programs should be literals?
    //For now, we assume all remaining literals are numbers
    //Recall strings are already captured
    var result = parseFloat(input);
    if(!isNaN(result)) {
        //Sucessfully parsed numeric literal
        return new ASTNumericLiteralNode(result);
    }

    return "ERROR: Could not parse literal \"" + input + "\"";
}

//This functions takes an AST and does any computation that can be done ahead of time, and returns the ASTNode representing it
//It recursively goes through and calculates whatever it can
function precompute_AST(ast, require_immediate=true) {
    //Process:
    //Take the current AST node. Go through each of it's parameters and check if it is available.
    //If it is, evaluate it and replace it with the relevant literal.
    //If not, call this function on it and replace the value with this.
    
    //If there are no parameters, there is nothing to do
    //Generally speaking, a 0 parameter AST node cannot be simplified
    //The only 0 parameter AST nodes are literals, and some functions, but those functions should still be evaluated
    if(ast.params.length == 0) {
        return ast;
    }

    if((require_immediate && ast.immediate_available()) || (!require_immediate && ast.available())) {
        //If we can calculate a value, we do so
        return create_AST_literal(ast.evaluate());
    }

    //If not, we check each parameter
    //We wont be able to make this available immediately but we can still precompute some things
    for(var i = 0;i < ast.params.length;i ++) {
        //All the logic is handled one level down
        ast.params[i] = precompute_AST(ast.params[i]);
    }

    return ast;//Return the AST with the modified values
}

//This function takes a single AST and attempts to evaluate it
//Note that pos is not a x, y pair or similar, like in eval_spreadsheet_formula(input, pos),
//it is a callback to assign the value where it should go
function evaluate_single(ast, pos) {
    if(ast.available()) {
        return ast.evaluate();
    } else {
        ast.make_available();
        computation_queue.push(new ASTJob(precompute_AST(ast), pos));
        return "AWAITING RESULT";
    }
}

//This function clears the cache and starts the evaluation process for everything over again
function mark_dirty() {

}

//This function attempts to re-evaluate all queued jobs
function mark_updated() {
    for(var i = 0;i < computation_queue.length;i ++) {
        if(computation_queue[i].ready_now()) {
            computation_queue[i].finish(computation_queue[i].evaluate());
            computation_queue.splice(i, 1);
            i--;//Need to backtrack
        } else {
            //Still awaiting result
        }
    }
}

//This functions take a formula from the spreadsheet, assembles it's AST, then evaluates it
//If it's not immediate, it gets sent to the queue and the callback is automatically set up
function eval_spreadsheet_formula(input, pos) {
    if(!input.startsWith("=")) {
        //Not a formula; ignore
        return input;
    }
    input = input.substring(1);//Remove the = sign

    //Creates the callback to set the value when completed
    var pos_cb = function(value) {
        var element = document.getElementById("spreadsheet-" + pos.i + ":" + pos.j);
        element.value = value;
    }
    var ast = create_AST(input, true);
    return evaluate_single(ast, pos_cb);
}