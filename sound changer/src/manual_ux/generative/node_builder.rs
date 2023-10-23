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
    compile_err_builder_node, compile_err_literal, compile_err_output_node, compile_err_typed_node,
    construction::{DataTypeDescriptor, EnumSpecifier, ProjectContext, TableColumnSpecifier},
    execution::{
        ColumnSpecifier, EnumNode, FilterPredicate, IntNode, OutputNode, RangeNode, StringNode,
        TableSpecifier, UIntNode,
    },
    CompileErrorType, GenerativeProgramCompileError,
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
    SoundChange,
    Mutate,
    SymbolLookup(String),
}

/// Represents an unchecked node. There can be severe type mismatch,
/// parameter count, etc. errors in this representation.
// TODO Add attribution information?
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum BuilderNode {
    TrueStringNode(StringNode),
    TrueIntNode(IntNode),
    TrueUIntNode(UIntNode),
    TrueEnumNode(EnumNode),
    TrueRangeNode(RangeNode),
    GenericLiteral(UnderspecifiedLiteral),
    FilterPredicate(FilterPredicate),
    /// Stores any node which is a combination of other builder nodes. No
    /// checking, not gaurunteed to be valid. The u8 value is precedence; used
    /// by `construction.rs` to implement operator precedence.
    CombinationNode(FunctionType, Vec<BuilderNode>, u8),
    Wrapper(Box<BuilderNode>),
}

/// Represents any node. Nodes are recursively generically converted to
/// this type, and then type checked as they are used.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TypedNode {
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
            // TODO: Verify enum type
            OutputNode::Enum(_) => match data_type {
                TableDataTypeDescriptor::Enum(_) => Ok(output_node),
                _ => {
                    compile_err_output_node(CompileErrorType::TypeMismatch("Got enum"), output_node)
                }
            },
            OutputNode::String(_) => match data_type {
                TableDataTypeDescriptor::String => Ok(output_node),
                _ => compile_err_output_node(
                    CompileErrorType::TypeMismatch("Got string"),
                    output_node,
                ),
            },
            OutputNode::Int(_) => match data_type {
                TableDataTypeDescriptor::Int => Ok(output_node),
                _ => {
                    compile_err_output_node(CompileErrorType::TypeMismatch("Got int"), output_node)
                }
            },
            OutputNode::UInt(_) => match data_type {
                TableDataTypeDescriptor::UInt => Ok(output_node),
                _ => {
                    compile_err_output_node(CompileErrorType::TypeMismatch("Got uint"), output_node)
                }
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
    // We allow thes issues because I think for the semantics it is expressing, the if chain is more
    // clear than the match statement.
    #[allow(clippy::comparison_chain, clippy::len_zero)]
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
            BuilderNode::CombinationNode(FunctionType::Filter, v, _) => {
                if v.len() != 2 {
                    panic!()
                }
                let range = v[0].clone().try_convert_range(context)?;
                let predicate = v[1].clone().try_convert_filter_predicate(context)?;
                Ok(TypedNode::RangeNode(RangeNode::FilterNode(
                    Box::new(range),
                    Box::new(predicate),
                )))
            }
            BuilderNode::CombinationNode(FunctionType::Foreach, v, p) => {
                if v.len() != 1 {
                    panic!()
                }
                let specifier = v[0].clone().try_convert_table_column(context)?;
                let (table, column) = match specifier {
                    TableColumnSpecifier::Table(_) => {
                        return compile_err_builder_node(
                            CompileErrorType::RequiresColumnSpecifier,
                            // Recreate node for attribution
                            BuilderNode::CombinationNode(FunctionType::Foreach, v, p),
                        );
                    }
                    TableColumnSpecifier::Column(_) => panic!(),
                    TableColumnSpecifier::Both(t, c) => (t, c),
                };
                Ok(TypedNode::RangeNode(RangeNode::ForeachNode(table, column)))
            }
            BuilderNode::CombinationNode(FunctionType::Save, v, _) => {
                if v.len() != 2 {
                    panic!()
                }
                let target = v[0].clone().try_convert_range(context)?;
                let name = v[1].clone().try_convert_string(context)?;
                Ok(TypedNode::RangeNode(RangeNode::Save(
                    Box::new(target),
                    Box::new(name),
                )))
            }
            BuilderNode::CombinationNode(FunctionType::Saved, v, p) => {
                if v.len() != 2 {
                    panic!()
                }
                let name = v[0].clone().try_convert_string(context)?;
                let table_column = v[1].clone().try_convert_table_column(context)?;
                let column = match table_column {
                    TableColumnSpecifier::Table(_) => {
                        return compile_err_builder_node(
                            CompileErrorType::RequiresColumnSpecifier,
                            // Recreate node for attribution
                            BuilderNode::CombinationNode(FunctionType::Saved, v, p),
                        );
                    }
                    TableColumnSpecifier::Column(c) => c,
                    TableColumnSpecifier::Both(_, c) => c,
                };
                Ok(TypedNode::RangeNode(RangeNode::Saved(
                    Box::new(name),
                    column,
                )))
            }
            BuilderNode::CombinationNode(FunctionType::SymbolLookup(name), v, p) => {
                if v.len() != 1 {
                    panic!()
                }
                let table_column_specifier = v[0].clone().try_convert_table_column(context)?;
                let (table_id, column_id) = match table_column_specifier {
                    TableColumnSpecifier::Table(_) => {
                        return compile_err_builder_node(
                            CompileErrorType::RequiresColumnSpecifier,
                            // Recreate node for attribution
                            BuilderNode::CombinationNode(FunctionType::SymbolLookup(name), v, p),
                        );
                    }
                    TableColumnSpecifier::Column(c) => (context.table_id, c.column_id),
                    TableColumnSpecifier::Both(t, c) => (t.table_id, c.column_id),
                };
                let data_type_descriptor = context.all_descriptors[&table_id].column_descriptors
                    [column_id]
                    .clone()
                    .data_type;
                match data_type_descriptor {
                    TableDataTypeDescriptor::Enum(v2) => {
                        let mut found_flag = false;
                        for s in v2 {
                            if s == name.to_lowercase() {
                                found_flag = true;
                            }
                        }
                        if !found_flag {
                            return compile_err_builder_node(
                                CompileErrorType::TypeMismatch("Couldn't find symbol"),
                                // Recreate node for attribution
                                BuilderNode::CombinationNode(
                                    FunctionType::SymbolLookup(name),
                                    v,
                                    p,
                                ),
                            );
                        }

                        Ok(TypedNode::EnumNode(EnumNode::LiteralNode(
                            name,
                            ColumnSpecifier { column_id },
                            TableSpecifier { table_id },
                        )))
                    }
                    _ => compile_err_builder_node(
                        CompileErrorType::TypeMismatch("Not an enum column"),
                        // Recreate node for attribution
                        BuilderNode::CombinationNode(FunctionType::SymbolLookup(name), v, p),
                    ),
                }
            }
            BuilderNode::CombinationNode(FunctionType::Addition, v, _) => {
                if v.len() != 2 {
                    panic!()
                }

                let data_type = match type_hint {
                    Some(DataTypeDescriptor::TableDataType(v)) => v,
                    _ => panic!(),
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
            BuilderNode::CombinationNode(FunctionType::SoundChange, v, _) => {
                if v.len() != 2 {
                    panic!()
                }

                let source = v[0].clone().try_convert_string(context)?;
                let program = v[1].clone().try_convert_string(context)?;

                Ok(TypedNode::StringNode(StringNode::SoundChangeNode(
                    Box::new(source),
                    Box::new(program),
                )))
            }
            BuilderNode::CombinationNode(FunctionType::Mutate, v, _) => {
                if v.len() != 3 {
                    panic!()
                }

                let a = v[0].clone().try_convert_range(context)?;
                let b = v[1].clone().try_convert_range(context)?;
                let mode = v[2].clone().try_convert_uint(context)?;

                Ok(TypedNode::RangeNode(RangeNode::Mutate(
                    Box::new(a),
                    Box::new(b),
                    Box::new(mode),
                )))
            }
            BuilderNode::Wrapper(v) => v.convert_to_node(context, type_hint),
            _ => {
                todo!()
            }
        }
    }
}

impl BuilderNode {
    /// This is a recursive function to insert a parameter into the current function call.<br><br>
    /// In certain cases with precedence, the top level node will not be the be the function: <br>
    /// i.e. consider `a + b.c(z) <=> a + c(b, z)` - when z is parsed, the top level node is the
    /// addition node `a + (b.c(z)). <br><br>
    /// This function typically doesn't need to go more than one level deep but it can go arbritrarily deep.
    /// It places the parameter into the first valid node it finds. Valid nodes are onces where
    /// `is_strictly_ordered()` returns true.
    pub fn insert_operand(&mut self, operand: BuilderNode) {
        match self {
            BuilderNode::CombinationNode(t, v, _) => {
                if t.is_strictly_ordered() {
                    v.push(operand)
                } else {
                    match t.final_operand_index() {
                        FinalOperandIndex::First => v[0].insert_operand(operand),
                        FinalOperandIndex::Last => {
                            let idx = v.len() - 1;
                            v[idx].insert_operand(operand);
                        }
                    }
                }
            }
            _ => panic!(),
        }
    }

    /// Finds the first strictly ordered node.<br>
    /// Similar to `insert_operand()`, more details there
    pub fn get_function_node(&mut self) -> &mut BuilderNode {
        if let BuilderNode::CombinationNode(t, _, _) = self {
            if t.is_strictly_ordered() {
                return self;
            }
        } else {
            return self;
        }

        if let BuilderNode::CombinationNode(t, v, _) = self {
            if !t.is_strictly_ordered() {
                match t.final_operand_index() {
                    FinalOperandIndex::First => return v[0].get_function_node(),
                    FinalOperandIndex::Last => {
                        let idx = v.len() - 1;
                        return v[idx].get_function_node();
                    }
                }
            }
        }

        unreachable!();
    }
}

impl TypedNode {
    fn try_convert_enum(self) -> Result<EnumNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::EnumNode(v) => Ok(v),
            TypedNode::RangeNode(v) => Ok(EnumNode::ConversionNode(v)),
            _ => compile_err_typed_node(CompileErrorType::TypeMismatch("Expected enum"), self),
        }
    }

    fn try_convert_string(self) -> Result<StringNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::StringNode(v) => Ok(v),
            TypedNode::RangeNode(v) => Ok(StringNode::ConversionNode(v)),
            _ => compile_err_typed_node(CompileErrorType::TypeMismatch("Expected string"), self),
        }
    }

    fn try_convert_uint(self) -> Result<UIntNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::UIntNode(v) => Ok(v),
            TypedNode::RangeNode(v) => Ok(UIntNode::ConversionNode(v)),
            _ => compile_err_typed_node(CompileErrorType::TypeMismatch("Expected uint"), self),
        }
    }

    fn try_convert_int(self) -> Result<IntNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::IntNode(v) => Ok(v),
            TypedNode::RangeNode(v) => Ok(IntNode::ConversionNode(v)),
            _ => compile_err_typed_node(CompileErrorType::TypeMismatch("Expected int"), self),
        }
    }

    fn try_convert_range(self) -> Result<RangeNode, GenerativeProgramCompileError> {
        match self {
            TypedNode::RangeNode(v) => Ok(v),
            _ => compile_err_typed_node(CompileErrorType::TypeMismatch("Expected range"), self),
        }
    }

    fn try_convert_filter_predicate(
        self,
    ) -> Result<FilterPredicate, GenerativeProgramCompileError> {
        match self {
            TypedNode::FilterPredicate(v) => Ok(v),
            _ => compile_err_typed_node(
                CompileErrorType::TypeMismatch("Expected filter predicate"),
                self,
            ),
        }
    }

    fn try_convert_table_column(
        self,
    ) -> Result<TableColumnSpecifier, GenerativeProgramCompileError> {
        match self {
            TypedNode::TableColumn(v) => Ok(v),
            _ => compile_err_typed_node(
                CompileErrorType::TypeMismatch("Expected table column specifier"),
                self,
            ),
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
            _ => compile_err_typed_node(
                CompileErrorType::TypeMismatch("Expected an output node"),
                self,
            ),
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
            RangeNode::Mutate(a, _, _) => {
                // TODO: Better data type heuristic
                a.get_data_type(context)
            }
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
            UnderspecifiedLiteral::Number(uint_value, int_value) => match type_hint {
                Some(DataTypeDescriptor::TableDataType(TableDataTypeDescriptor::Int)) => {
                    Ok(TypedNode::IntNode(IntNode::LiteralNode(int_value)))
                }
                Some(DataTypeDescriptor::TableDataType(TableDataTypeDescriptor::UInt)) => {
                    Ok(TypedNode::UIntNode(UIntNode::LiteralNode(uint_value)))
                }
                None => panic!(),
                _ => todo!(),
            },
            UnderspecifiedLiteral::TableColumnSpecifier(v) => Ok(TypedNode::TableColumn(v)),
            UnderspecifiedLiteral::StringOrShortEnum(string, c, t) => match type_hint {
                Some(DataTypeDescriptor::TableDataType(TableDataTypeDescriptor::Enum(_))) => {
                    todo!()
                }
                None | Some(DataTypeDescriptor::TableDataType(TableDataTypeDescriptor::String)) => {
                    Ok(TypedNode::StringNode(StringNode::LiteralNode(string)))
                }
                _ => compile_err_literal(
                    CompileErrorType::TypeMismatch("Invalid type hint for node type"),
                    UnderspecifiedLiteral::StringOrShortEnum(string, c, t),
                ),
            },
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Copy)]
pub enum FinalOperandIndex {
    First,
    Last,
}

impl FunctionType {
    /// A node is strictly ordered if it is non-communative and the simple
    /// ordering scheme does not work.
    /// i.e. `1 + 2 * 3 / 4 - 5` is not strictly ordered because the simple
    /// ordering scheme will generate a correct configuration (`((1 + ((2 * 3) / 4)) - 5)`).
    /// i.e `a + b.c(z)` is strictly ordered because the function call operation requires
    /// the special handling.
    fn is_strictly_ordered(&self) -> bool {
        match self {
            FunctionType::UnknownFunction
            | FunctionType::Foreach
            | FunctionType::Filter
            | FunctionType::Save
            | FunctionType::Saved
            | FunctionType::SoundChange
            | FunctionType::Mutate
            | FunctionType::SymbolLookup(_) => true,
            FunctionType::Addition | FunctionType::Subtraction => false,
        }
    }

    pub fn final_operand_index(&self) -> FinalOperandIndex {
        match self {
            FunctionType::UnknownFunction
            | FunctionType::Filter
            | FunctionType::Save
            | FunctionType::SoundChange
            | FunctionType::SymbolLookup(_)
            | FunctionType::Mutate => FinalOperandIndex::First,
            FunctionType::Addition | FunctionType::Subtraction => FinalOperandIndex::Last,
            FunctionType::Saved | FunctionType::Foreach => unreachable!(),
        }
    }
}
