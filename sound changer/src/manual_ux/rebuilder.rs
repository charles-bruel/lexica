use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

use crate::{
    io,
    manual_ux::{
        generative::{
            execution::GenerativeProgramExecutionOutput, runtime_err,
            GenerativeProgramRuntimeError, RuntimeErrorType,
        },
        table::{PopulatedTableRowSource, TableContents},
    },
};

use super::{
    generative::execution::{ExecutionContext, TableSpecifier},
    project::Project,
    table::{Table, TableRow},
};

pub fn rebuild(project: &mut Project, start: u16, base_path: String, do_io: bool) {
    let mut index = start as usize;
    while index < project.tables.len() {
        if let Some(mut table) = project.tables[index].clone() {
            table.reset();
            table.rebuild(project, base_path.clone(), do_io).unwrap();

            project.tables[index] = Some(table);
        }

        index += 1;
    }
}

impl Table {
    fn reset(&mut self) {
        // Replaces table_rows with src_table_rows
        self.table_rows = self.src_table_rows.clone();
    }

    fn rebuild(
        &mut self,
        project: &mut Project,
        base_path: String,
        do_io: bool,
    ) -> Result<(), GenerativeProgramRuntimeError> {
        // 1) Execute all unpopulated table rows

        let start = Instant::now();

        let new_rows = Vec::with_capacity(self.table_rows.len());
        let old_rows = VecDeque::from(std::mem::replace(&mut self.table_rows, new_rows));

        for row in old_rows {
            match row {
                TableRow::PopulatedTableRow {
                    source,
                    descriptor,
                    contents,
                } => {
                    let new_row = TableRow::PopulatedTableRow {
                        source,
                        descriptor,
                        contents,
                    };
                    self.table_rows.push(new_row);
                }
                TableRow::UnpopulatedTableRow {
                    procedure,
                    descriptor,
                } => {
                    // Run row
                    let mut context: ExecutionContext = ExecutionContext {
                        saved_ranges: HashMap::new(),
                        table_descriptor: descriptor.clone(),
                        table_specifer: TableSpecifier {
                            table_id: self.id as usize,
                        },
                        project,
                        base_path: base_path.clone(),
                    };
                    let mut results = Vec::with_capacity(procedure.programs.len());
                    for column in &procedure.programs {
                        results.push(column.evaluate(&mut context).unwrap());
                    }
                    assert_eq!(
                        results.len(),
                        context.table_descriptor.column_descriptors.len()
                    );

                    // Determine number of rows
                    let mut first_not_one_count = None;
                    for result in &results {
                        if result.len() != 1 {
                            match first_not_one_count {
                                None => first_not_one_count = Some(result.len()),
                                Some(c) => {
                                    if c != result.len() {
                                        return runtime_err(
                                            RuntimeErrorType::MismatchedRangeLengths,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    let count = first_not_one_count.unwrap_or(1);

                    // Put results into table
                    let mut final_result = Vec::with_capacity(count);
                    let mut index = 0;
                    let table_row_source = PopulatedTableRowSource::CACHE(procedure.clone());
                    while index < count {
                        let mut contents = Vec::with_capacity(results.len());
                        for (i, result) in results.iter().enumerate() {
                            let actual_index = if result.len() == 1 { 0 } else { index };
                            match result {
                                GenerativeProgramExecutionOutput::String(v) => {
                                    descriptor.column_descriptors[i].data_type.assert_string();
                                    let table_contents =
                                        TableContents::String(v[actual_index].clone());
                                    contents.push(table_contents);
                                }
                                GenerativeProgramExecutionOutput::Int(v) => {
                                    descriptor.column_descriptors[i].data_type.assert_int();
                                    let table_contents = TableContents::Int(v[actual_index]);
                                    contents.push(table_contents);
                                }
                                GenerativeProgramExecutionOutput::UInt(v) => {
                                    descriptor.column_descriptors[i].data_type.assert_uint();
                                    let table_contents = TableContents::UInt(v[actual_index]);
                                    contents.push(table_contents);
                                }
                                GenerativeProgramExecutionOutput::Enum(v) => {
                                    // TODO: Check specific enum type
                                    descriptor.column_descriptors[i].data_type.assert_enum();
                                    let table_contents =
                                        TableContents::Enum(v[actual_index].clone());
                                    contents.push(table_contents);
                                }
                            }
                        }

                        let table_row = TableRow::PopulatedTableRow {
                            source: table_row_source.clone(),
                            descriptor: descriptor.clone(),
                            contents,
                        };
                        final_result.push(table_row);
                        index += 1;
                    }

                    self.table_rows.append(&mut final_result);
                }
            }
        }

        let elapsed = start.elapsed();

        if do_io {
            // 2) Output
            println!();
            let output = self.output(project);
            println!("{}", output);
            let path_str = self.source_path.clone() + ".out";
            println!("Rebuilt table in {:?}", elapsed);

            io::save_to_file(&path_str, &output, true, false);
        }

        Ok(())
    }
}
