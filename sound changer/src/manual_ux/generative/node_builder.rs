// Underspecified versions of the nodes found in execution.rs.
// This is an intermediate step between the linear code and the
// tree AST - it is a tree optimized to be build linearly.
// This allows the structure to be build piece by piece without
// weakening or overcomplicating the execution data types.
// All nodes are treated as interchangeable.
// If possible, the final, strong, "true" versions of each node
// will be used.

use crate::manual_ux::table::TableDataTypeDescriptor;

use super::{execution::{RangeNode, StringNode, IntNode, UIntNode, EnumNode, FilterPredicate, OutputNode, TableSpecifier, ColumnSpecifier, Range}, construction::{DataTypeDescriptor, EnumSpecifier, TableColumnSpecifier, ProjectContext}, GenerativeProgramCompileError};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum UnderspecifiedLiteral {
    String(String),
    Enum(EnumSpecifier),
    Int(i32),
    UInt(u32),
    Number(u32, i32),
    TableColumnSpecifier(TableColumnSpecifier),
    StringOrShortEnum(String, ColumnSpecifier, TableSpecifier),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FunctionType {
    UnknownFunction,
    Addition,
    Conversion,
    Foreach,
    Filter,
    Save,
    Saved,
    SymbolLookup(String),
}

/// Represents an unchecked node. There can be severe type mismatch,
/// parameter count, etc. errors in this representation.
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

/// Represents any node. Nodes are recursively generically converted to
/// this type, and then type checked as they are used.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum TypedNode {
    StringNode(StringNode),
    IntNode(IntNode),
    UIntNode(UIntNode),
    EnumNode(EnumNode),
    RangeNode(RangeNode),
    TableColumn(TableColumnSpecifier),
    FilterPredicate(FilterPredicate),
}

impl BuilderNode {
    pub fn try_convert_output_node(
        self,
        data_type: &DataTypeDescriptor, 
        context: &ProjectContext,
    ) -> Result<OutputNode, GenerativeProgramCompileError> {
        todo!() // Check data type
        // self.convert_to_node(context)?.try_convert_output_node()
    }

    pub fn try_convert_enum(self, context: &ProjectContext,) -> Result<EnumNode, GenerativeProgramCompileError> {
        self.convert_to_node(context)?.try_convert_enum()
    }

    pub fn try_convert_string(self, context: &ProjectContext,) -> Result<StringNode, GenerativeProgramCompileError> {
        self.convert_to_node(context)?.try_convert_string()
    }

    pub fn try_convert_int(self, context: &ProjectContext,) -> Result<IntNode, GenerativeProgramCompileError> {
        self.convert_to_node(context)?.try_convert_int()
    }

    pub fn try_convert_uint(self, context: &ProjectContext,) -> Result<UIntNode, GenerativeProgramCompileError> {
        self.convert_to_node(context)?.try_convert_uint()
    }

    pub fn try_convert_range(self, context: &ProjectContext,) -> Result<RangeNode, GenerativeProgramCompileError> {
        self.convert_to_node(context)?.try_convert_range()
    }

    pub fn try_convert_filter_predicate(self, context: &ProjectContext,) -> Result<FilterPredicate, GenerativeProgramCompileError> {
        self.convert_to_node(context)?.try_convert_filter_predicate()
    }

    pub fn try_convert_table_column(self, context: &ProjectContext,) -> Result<TableColumnSpecifier, GenerativeProgramCompileError> {
        self.convert_to_node(context)?.try_convert_table_column()
    }

    fn convert_to_node(self, context: &ProjectContext) -> Result<TypedNode, GenerativeProgramCompileError> {
        match self {
            // Easy
            BuilderNode::TrueStringNode(v) => Ok(TypedNode::StringNode(v)),
            BuilderNode::TrueIntNode(v) => Ok(TypedNode::IntNode(v)),
            BuilderNode::TrueUIntNode(v) => Ok(TypedNode::UIntNode(v)),
            BuilderNode::TrueEnumNode(v) => Ok(TypedNode::EnumNode(v)),
            BuilderNode::TrueRangeNode(v) => Ok(TypedNode::RangeNode(v)),
            BuilderNode::GenericLiteral(v) => Ok(v.convert(context)),
            BuilderNode::FilterPredicate(v) => Ok(TypedNode::FilterPredicate(v)),

            // Hard
            BuilderNode::CombinationNode(FunctionType::Filter, v) => {
                if v.len() < 2 {
                    todo!()
                } else if v.len() > 2 {
                    todo!()
                }
                let range = v[0].clone().try_convert_range(context)?;
                let predicate = v[1].clone().try_convert_filter_predicate(context)?;
                Ok(TypedNode::RangeNode(RangeNode::FilterNode(Box::new(range), Box::new(predicate))))
            },
            BuilderNode::CombinationNode(FunctionType::Foreach, v) => {
                if v.len() < 1 {
                    todo!()
                } else if v.len() > 1 {
                    todo!()
                }
                let specifier = v[0].clone().try_convert_table_column(context)?;
                let (table, column) = match specifier {
                    TableColumnSpecifier::TABLE(_) => return Err(GenerativeProgramCompileError::RequiresColumnSpecifier),
                    TableColumnSpecifier::COLUMN(_) => todo!(),
                    TableColumnSpecifier::BOTH(t, c) => (t, c),
                };
                Ok(TypedNode::RangeNode(RangeNode::ForeachNode(table, column)))
            },
            BuilderNode::CombinationNode(FunctionType::SymbolLookup(name), v) => {
                if v.len() < 1 {
                    todo!()
                } else if v.len() > 1 {
                    todo!()
                }
                let table_column_specifier = v[0].clone().try_convert_table_column(context)?;
                let (table_id, column_id) = match table_column_specifier {
                    TableColumnSpecifier::TABLE(_) => return Err(GenerativeProgramCompileError::RequiresColumnSpecifier),
                    TableColumnSpecifier::COLUMN(c) => (context.table_id, c.column_id),
                    TableColumnSpecifier::BOTH(t, c) => (t.table_id, c.column_id),
                };
                let data_type_descriptor = context.all_descriptors[&table_id].column_descriptors[column_id].clone().data_type;
                match data_type_descriptor {
                    TableDataTypeDescriptor::Enum(v) => {
                        let mut found_flag = false;
                        for s in v {
                            if s == name.to_lowercase() {
                                found_flag = true;
                            }
                        }
                        if !found_flag {
                            return  Err(GenerativeProgramCompileError::TypeMismatch);
                        }


                        Ok(TypedNode::EnumNode(EnumNode::LiteralNode(name, ColumnSpecifier { column_id }, TableSpecifier { table_id })))
                    },
                    _ => Err(GenerativeProgramCompileError::TypeMismatch),
                }
            }
            _ => {
                println!("{:?}", self);
                todo!()
            },
        }
    }
}

impl TypedNode {
    fn try_convert_enum(self) -> Result<EnumNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::EnumNode(v) => Ok(v),
            _ => Err(GenerativeProgramCompileError::TypeMismatch)
        }
    }

    fn try_convert_string(self) -> Result<StringNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::StringNode(v) => Ok(v),
            _ => Err(GenerativeProgramCompileError::TypeMismatch)
        }
    }

    fn try_convert_uint(self) -> Result<UIntNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::UIntNode(v) => Ok(v),
            _ => Err(GenerativeProgramCompileError::TypeMismatch)
        }
    }

    fn try_convert_int(self) -> Result<IntNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::IntNode(v) => Ok(v),
            _ => Err(GenerativeProgramCompileError::TypeMismatch)
        }
    }

    fn try_convert_range(self) -> Result<RangeNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::RangeNode(v) => Ok(v),
            _ => Err(GenerativeProgramCompileError::TypeMismatch)
        }
    }

    fn try_convert_filter_predicate(self) -> Result<FilterPredicate, GenerativeProgramCompileError> {
        match self {
            TypedNode::FilterPredicate(v) => Ok(v),
            _ => Err(GenerativeProgramCompileError::TypeMismatch)
        }
    }

    fn try_convert_table_column(self) -> Result<TableColumnSpecifier, GenerativeProgramCompileError> {
        match self {
            TypedNode::TableColumn(v) => Ok(v),
            _ => Err(GenerativeProgramCompileError::TypeMismatch)
        }
    }


    fn try_convert_output_node(self) -> Result<OutputNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::StringNode(v) => Ok(OutputNode::String(v)),
            TypedNode::IntNode(v) => Ok(OutputNode::Int(v)),
            TypedNode::UIntNode(v) => Ok(OutputNode::UInt(v)),
            TypedNode::EnumNode(v) => Ok(OutputNode::Enum(v)),
            _ => Err(GenerativeProgramCompileError::TypeMismatch),
        }
    }
}

impl UnderspecifiedLiteral {
    // TODO: Specific types
    fn convert(self, context: &ProjectContext) -> TypedNode {
        match self {
            UnderspecifiedLiteral::String(v) => TypedNode::StringNode(StringNode::LiteralNode(v)),
            UnderspecifiedLiteral::Enum(v) => {
                let table = match v.table {
                    Some(v) => v,
                    None => TableSpecifier { table_id: context.table_id },
                };

                TypedNode::EnumNode(EnumNode::LiteralNode(v.name, v.column, table))
            },
            UnderspecifiedLiteral::Int(_) => todo!(),
            UnderspecifiedLiteral::UInt(_) => todo!(),
            UnderspecifiedLiteral::Number(_, _) => todo!(),
            UnderspecifiedLiteral::TableColumnSpecifier(v) => TypedNode::TableColumn(v),
            UnderspecifiedLiteral::StringOrShortEnum(_, _, _) => todo!(),
        }
    }
}