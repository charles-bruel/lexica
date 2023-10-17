// Underspecified versions of the nodes found in execution.rs.
// This is an intermediate step between the linear code and the
// tree AST - it is a tree optimized to be build linearly.
// This allows the structure to be build piece by piece without
// weakening or overcomplicating the execution data types.
// All nodes are treated as interchangeable.
// If possible, the final, strong, "true" versions of each node
// will be used.

use crate::manual_ux::table::TableDataTypeDescriptor;

use super::{
    construction::{DataTypeDescriptor, EnumSpecifier, ProjectContext, TableColumnSpecifier},
    execution::{
        ColumnSpecifier, EnumNode, FilterPredicate, IntNode, OutputNode, RangeNode, StringNode,
        TableSpecifier, UIntNode,
    },
    GenerativeProgramCompileError,
};

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
    // We skip conversion nodes here because those are constructed
    // during this step.
    UnknownFunction,
    Addition,
    Subtraction,
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
        data_type: &TableDataTypeDescriptor,
        context: &ProjectContext,
    ) -> Result<OutputNode, GenerativeProgramCompileError> {
        let output_node = self
            .convert_to_node(
                context,
                &Some(DataTypeDescriptor::TableDataType(data_type.clone())),
            )?
            .try_convert_output_node(context)?;
        match output_node {
            OutputNode::String(_) => match data_type {
                TableDataTypeDescriptor::String => Ok(output_node),
                _ => Err(GenerativeProgramCompileError::TypeMismatch("Got string")),
            },
            OutputNode::Int(_) => match data_type {
                TableDataTypeDescriptor::Int => Ok(output_node),
                _ => Err(GenerativeProgramCompileError::TypeMismatch("Got int")),
            },
            OutputNode::UInt(_) => match data_type {
                TableDataTypeDescriptor::UInt => Ok(output_node),
                _ => Err(GenerativeProgramCompileError::TypeMismatch("Got uint")),
            },
            // TODO: Verify enum type
            OutputNode::Enum(_) => match data_type {
                TableDataTypeDescriptor::Enum(_) => Ok(output_node),
                _ => Err(GenerativeProgramCompileError::TypeMismatch("Got enum")),
            },
        }
    }

    pub fn try_convert_enum(
        self,
        context: &ProjectContext,
    ) -> Result<EnumNode, GenerativeProgramCompileError> {
        // TODO: Enum data types in type hints
        let type_hint = &Some(DataTypeDescriptor::TableDataType(
            TableDataTypeDescriptor::Enum(vec![]),
        ));
        self.convert_to_node(context, type_hint)?.try_convert_enum()
    }

    pub fn try_convert_string(
        self,
        context: &ProjectContext,
    ) -> Result<StringNode, GenerativeProgramCompileError> {
        let type_hint = &Some(DataTypeDescriptor::TableDataType(
            TableDataTypeDescriptor::String,
        ));
        self.convert_to_node(context, type_hint)?
            .try_convert_string()
    }

    pub fn try_convert_int(
        self,
        context: &ProjectContext,
    ) -> Result<IntNode, GenerativeProgramCompileError> {
        let type_hint = &Some(DataTypeDescriptor::TableDataType(
            TableDataTypeDescriptor::Int,
        ));
        self.convert_to_node(context, type_hint)?.try_convert_int()
    }

    pub fn try_convert_uint(
        self,
        context: &ProjectContext,
    ) -> Result<UIntNode, GenerativeProgramCompileError> {
        let type_hint = &Some(DataTypeDescriptor::TableDataType(
            TableDataTypeDescriptor::UInt,
        ));
        self.convert_to_node(context, type_hint)?.try_convert_uint()
    }

    pub fn try_convert_range(
        self,
        context: &ProjectContext,
    ) -> Result<RangeNode, GenerativeProgramCompileError> {
        let type_hint = &Some(DataTypeDescriptor::Range);
        self.convert_to_node(context, type_hint)?
            .try_convert_range()
    }

    pub fn try_convert_filter_predicate(
        self,
        context: &ProjectContext,
    ) -> Result<FilterPredicate, GenerativeProgramCompileError> {
        let type_hint = &Some(DataTypeDescriptor::FilterPredicate);
        self.convert_to_node(context, type_hint)?
            .try_convert_filter_predicate()
    }

    pub fn try_convert_table_column(
        self,
        context: &ProjectContext,
    ) -> Result<TableColumnSpecifier, GenerativeProgramCompileError> {
        let type_hint = &Some(DataTypeDescriptor::TableColumnSpecifier);
        self.convert_to_node(context, type_hint)?
            .try_convert_table_column()
    }

    /// Note that type hints are just hints, and are only sometimes used. If you want strict
    /// typing, check the output of this function yourself.
    fn convert_to_node(
        self,
        context: &ProjectContext,
        type_hint: &Option<DataTypeDescriptor>,
    ) -> Result<TypedNode, GenerativeProgramCompileError> {
        match self {
            // Easy
            BuilderNode::TrueStringNode(v) => Ok(TypedNode::StringNode(v)),
            BuilderNode::TrueIntNode(v) => Ok(TypedNode::IntNode(v)),
            BuilderNode::TrueUIntNode(v) => Ok(TypedNode::UIntNode(v)),
            BuilderNode::TrueEnumNode(v) => Ok(TypedNode::EnumNode(v)),
            BuilderNode::TrueRangeNode(v) => Ok(TypedNode::RangeNode(v)),
            BuilderNode::GenericLiteral(v) => Ok(v.convert(context, type_hint)?),
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
                Ok(TypedNode::RangeNode(RangeNode::FilterNode(
                    Box::new(range),
                    Box::new(predicate),
                )))
            }
            BuilderNode::CombinationNode(FunctionType::Foreach, v) => {
                if v.len() < 1 {
                    todo!()
                } else if v.len() > 1 {
                    todo!()
                }
                let specifier = v[0].clone().try_convert_table_column(context)?;
                let (table, column) = match specifier {
                    TableColumnSpecifier::TABLE(_) => {
                        return Err(GenerativeProgramCompileError::RequiresColumnSpecifier)
                    }
                    TableColumnSpecifier::COLUMN(_) => todo!(),
                    TableColumnSpecifier::BOTH(t, c) => (t, c),
                };
                Ok(TypedNode::RangeNode(RangeNode::ForeachNode(table, column)))
            }
            BuilderNode::CombinationNode(FunctionType::Save, v) => {
                if v.len() < 2 {
                    todo!()
                } else if v.len() > 2 {
                    todo!()
                }
                let target = v[0].clone().try_convert_range(context)?;
                let name = v[1].clone().try_convert_string(context)?;
                Ok(TypedNode::RangeNode(RangeNode::Save(
                    Box::new(target),
                    Box::new(name),
                )))
            }
            BuilderNode::CombinationNode(FunctionType::Saved, v) => {
                if v.len() < 2 {
                    todo!()
                } else if v.len() > 2 {
                    todo!()
                }
                let name = v[0].clone().try_convert_string(context)?;
                let table_column = v[1].clone().try_convert_table_column(context)?;
                let column = match table_column {
                    TableColumnSpecifier::TABLE(_) => {
                        return Err(GenerativeProgramCompileError::RequiresColumnSpecifier)
                    }
                    TableColumnSpecifier::COLUMN(c) => c,
                    TableColumnSpecifier::BOTH(_, c) => c,
                };
                Ok(TypedNode::RangeNode(RangeNode::Saved(
                    Box::new(name),
                    column,
                )))
            }
            BuilderNode::CombinationNode(FunctionType::SymbolLookup(name), v) => {
                if v.len() < 1 {
                    todo!()
                } else if v.len() > 1 {
                    todo!()
                }
                let table_column_specifier = v[0].clone().try_convert_table_column(context)?;
                let (table_id, column_id) = match table_column_specifier {
                    TableColumnSpecifier::TABLE(_) => {
                        return Err(GenerativeProgramCompileError::RequiresColumnSpecifier)
                    }
                    TableColumnSpecifier::COLUMN(c) => (context.table_id, c.column_id),
                    TableColumnSpecifier::BOTH(t, c) => (t.table_id, c.column_id),
                };
                let data_type_descriptor = context.all_descriptors[&table_id].column_descriptors
                    [column_id]
                    .clone()
                    .data_type;
                match data_type_descriptor {
                    TableDataTypeDescriptor::Enum(v) => {
                        let mut found_flag = false;
                        for s in v {
                            if s == name.to_lowercase() {
                                found_flag = true;
                            }
                        }
                        if !found_flag {
                            return Err(GenerativeProgramCompileError::TypeMismatch(
                                "Couldn't find symbol",
                            ));
                        }

                        Ok(TypedNode::EnumNode(EnumNode::LiteralNode(
                            name,
                            ColumnSpecifier { column_id },
                            TableSpecifier { table_id },
                        )))
                    }
                    _ => Err(GenerativeProgramCompileError::TypeMismatch(
                        "Not an enum column",
                    )),
                }
            }
            BuilderNode::CombinationNode(FunctionType::Addition, v) => {
                if v.len() < 2 {
                    todo!()
                } else if v.len() > 2 {
                    todo!()
                }

                let data_type = match type_hint {
                    Some(DataTypeDescriptor::TableDataType(v)) => v,
                    _ => todo!(),
                };

                match data_type {
                    TableDataTypeDescriptor::Enum(_) => todo!(),
                    TableDataTypeDescriptor::String => {
                        Ok(TypedNode::StringNode(StringNode::AdditionNode(
                            Box::new(v[0].clone().try_convert_string(context)?),
                            Box::new(v[1].clone().try_convert_string(context)?),
                        )))
                    }
                    TableDataTypeDescriptor::UInt => {
                        Ok(TypedNode::UIntNode(UIntNode::AdditionNode(
                            Box::new(v[0].clone().try_convert_uint(context)?),
                            Box::new(v[1].clone().try_convert_uint(context)?),
                        )))
                    }
                    TableDataTypeDescriptor::Int => Ok(TypedNode::IntNode(IntNode::AdditionNode(
                        Box::new(v[0].clone().try_convert_int(context)?),
                        Box::new(v[1].clone().try_convert_int(context)?),
                    ))),
                }
            }
            _ => {
                println!("{:?}", self);
                todo!()
            }
        }
    }
}

impl TypedNode {
    fn try_convert_enum(self) -> Result<EnumNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::EnumNode(v) => Ok(v),
            TypedNode::RangeNode(v) => Ok(EnumNode::ConversionNode(v)),
            _ => Err(GenerativeProgramCompileError::TypeMismatch("Expected enum")),
        }
    }

    fn try_convert_string(self) -> Result<StringNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::StringNode(v) => Ok(v),
            TypedNode::RangeNode(v) => Ok(StringNode::ConversionNode(v)),
            _ => Err(GenerativeProgramCompileError::TypeMismatch(
                "Expected string",
            )),
        }
    }

    fn try_convert_uint(self) -> Result<UIntNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::UIntNode(v) => Ok(v),
            TypedNode::RangeNode(v) => Ok(UIntNode::ConversionNode(v)),
            _ => Err(GenerativeProgramCompileError::TypeMismatch("Expected uint")),
        }
    }

    fn try_convert_int(self) -> Result<IntNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::IntNode(v) => Ok(v),
            TypedNode::RangeNode(v) => Ok(IntNode::ConversionNode(v)),
            _ => Err(GenerativeProgramCompileError::TypeMismatch("Expected int")),
        }
    }

    fn try_convert_range(self) -> Result<RangeNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::RangeNode(v) => Ok(v),
            _ => Err(GenerativeProgramCompileError::TypeMismatch(
                "Expected range",
            )),
        }
    }

    fn try_convert_filter_predicate(
        self,
    ) -> Result<FilterPredicate, GenerativeProgramCompileError> {
        match self {
            TypedNode::FilterPredicate(v) => Ok(v),
            _ => Err(GenerativeProgramCompileError::TypeMismatch(
                "Expected filter predicate",
            )),
        }
    }

    fn try_convert_table_column(
        self,
    ) -> Result<TableColumnSpecifier, GenerativeProgramCompileError> {
        match self {
            TypedNode::TableColumn(v) => Ok(v),
            _ => Err(GenerativeProgramCompileError::TypeMismatch(
                "Expected table column specifier",
            )),
        }
    }

    fn try_convert_output_node(
        self,
        context: &ProjectContext,
    ) -> Result<OutputNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::StringNode(v) => Ok(OutputNode::String(v)),
            TypedNode::IntNode(v) => Ok(OutputNode::Int(v)),
            TypedNode::UIntNode(v) => Ok(OutputNode::UInt(v)),
            TypedNode::EnumNode(v) => Ok(OutputNode::Enum(v)),
            TypedNode::RangeNode(v) => {
                let data_type = v.get_data_type(context);

                match data_type {
                    TableDataTypeDescriptor::Enum(_) => {
                        Ok(OutputNode::Enum(EnumNode::ConversionNode(v)))
                    }
                    TableDataTypeDescriptor::String => {
                        Ok(OutputNode::String(StringNode::ConversionNode(v)))
                    }
                    TableDataTypeDescriptor::UInt => {
                        Ok(OutputNode::UInt(UIntNode::ConversionNode(v)))
                    }
                    TableDataTypeDescriptor::Int => Ok(OutputNode::Int(IntNode::ConversionNode(v))),
                }
            }
            _ => Err(GenerativeProgramCompileError::TypeMismatch(
                "Expected an output node",
            )),
        }
    }
}

impl RangeNode {
    fn get_data_type(&self, context: &ProjectContext) -> TableDataTypeDescriptor {
        match self {
            RangeNode::FilterNode(prev, _) | RangeNode::Save(prev, _) => {
                prev.get_data_type(context)
            }
            RangeNode::Saved(_, column) => context.descriptor.column_descriptors[column.column_id]
                .data_type
                .clone(),
            RangeNode::ForeachNode(table, column) => context.all_descriptors[&table.table_id]
                .column_descriptors[column.column_id]
                .data_type
                .clone(),
        }
    }
}

impl UnderspecifiedLiteral {
    // TODO: Specific types
    fn convert(
        self,
        context: &ProjectContext,
        type_hint: &Option<DataTypeDescriptor>,
    ) -> Result<TypedNode, GenerativeProgramCompileError> {
        match self {
            UnderspecifiedLiteral::String(v) => {
                Ok(TypedNode::StringNode(StringNode::LiteralNode(v)))
            }
            UnderspecifiedLiteral::Enum(v) => {
                let table = match v.table {
                    Some(v) => v,
                    None => TableSpecifier {
                        table_id: context.table_id,
                    },
                };

                Ok(TypedNode::EnumNode(EnumNode::LiteralNode(
                    v.name, v.column, table,
                )))
            }
            UnderspecifiedLiteral::Int(_) => todo!(),
            UnderspecifiedLiteral::UInt(_) => todo!(),
            UnderspecifiedLiteral::Number(_, _) => todo!(),
            UnderspecifiedLiteral::TableColumnSpecifier(v) => Ok(TypedNode::TableColumn(v)),
            UnderspecifiedLiteral::StringOrShortEnum(string, _column, _table) => match type_hint {
                Some(DataTypeDescriptor::TableDataType(TableDataTypeDescriptor::Enum(_))) => {
                    todo!()
                }
                None | Some(DataTypeDescriptor::TableDataType(TableDataTypeDescriptor::String)) => {
                    Ok(TypedNode::StringNode(StringNode::LiteralNode(string)))
                }
                _ => Err(GenerativeProgramCompileError::TypeMismatch(
                    "Invalid type hint for node type",
                )),
            },
        }
    }
}
