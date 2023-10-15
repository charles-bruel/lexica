// Underspecified versions of the nodes found in execution.rs.
// This is an intermediate step between the linear code and the
// tree AST - it is a tree optimized to be build linearly.
// This allows the structure to be build piece by piece without
// weakening or overcomplicating the execution data types.
// All nodes are treated as interchangeable.
// If possible, the final, strong, "true" versions of each node
// will be used.

use super::{execution::{RangeNode, StringNode, IntNode, UIntNode, EnumNode, FilterPredicate, OutputNode}, construction::{UnderspecifiedLiteral, DataTypeDescriptor}, GenerativeProgramCompileError};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum FunctionType {
    Addition,
    Conversion,
    Foreach,
    Filter,
    Save,
    Saved,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum BuilderNode {
    TrueStringNode(StringNode),
    TrueIntNode(IntNode),
    TrueUIntNode(UIntNode),
    TrueEnumNode(EnumNode),
    TrueRangeNode(RangeNode),
    GenericLiteral(UnderspecifiedLiteral),
    FilterPredicate(FilterPredicate),
    CombinationNode(FunctionType, Vec<BuilderNode>),
}

impl BuilderNode {
    pub fn try_convert_output_node(
        self,
        data_type: &DataTypeDescriptor,
    ) -> Result<OutputNode, GenerativeProgramCompileError> {
        todo!()
    }
}