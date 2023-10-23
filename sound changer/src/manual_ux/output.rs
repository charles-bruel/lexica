use tabled::{builder::Builder, settings::Style};

use super::{table::{Table, TableRow}, project::Project};

impl Table {
    pub fn output(&self, project: &Project) -> String {
        let mut builder = Builder::default();

        let mut headers = Vec::new();
        for column in &self.table_descriptor.column_descriptors {
            headers.push(column.name.clone());
        }
        builder.set_header(headers);

        for row in &self.table_rows {
            if let TableRow::PopulatedTableRow {
                source: _,
                descriptor: _,
                contents,
            } = row {
                let mut strings = Vec::new();
                for content in contents {
                    strings.push(content.to_string(project));
                }
                builder.push_record(strings);
            }
        }

        let mut table = builder.build();
        table.with(Style::markdown()).to_string()
    }
}