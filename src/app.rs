use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use thiserror::Error;
use eyre::{WrapErr, Result};
use log::trace;
use crate::columns::Columns;
use crate::table::TableBuilder;
use crate::core::TableQuotient;

#[derive(Error,Debug)]
pub enum Error {
    #[error("Only one standard input is acceptable")]
    MultipleStdin,
}

/// Compare two CSV files 
#[derive(Debug,StructOpt)]
pub struct App {
    /// Path to a CSV file
    ///
    /// If multiple files are given, their contents are concatenated.
    /// The sign `-` means the standard input.
    #[structopt(long)]
    source_file: Vec<PathBuf>,
    /// Path to a CSV file
    ///
    /// If multiple files are given, their contents are concatenated.
    /// The sign `-` means the standard input.
    #[structopt(long)]
    target_file: Vec<PathBuf>,
    /// Define of source file columns
    #[structopt(long)]
    source_columns: String,
    /// Define target file columns
    #[structopt(long)]
    target_columns: String,
    /// Determine whether records are same or not
    #[structopt(long)]
    predicate: String,
    /// Ignore field length mismatches
    #[structopt(long)]
    ignore_length_mismatch: bool,
    /// Ignore parse errors for fields
    #[structopt(long)]
    ignore_parse_error: bool,
}

impl App {
    fn validate_path(&self) -> Result<(), Error> {
        let hyphen = PathBuf::from_str("-").unwrap();
        let src_hyphens = self.source_file
            .iter()
            .filter(|path| *path == &hyphen)
            .count();
        let dst_hyphens = self.target_file
            .iter()
            .filter(|path| *path == &hyphen)
            .count();
        if src_hyphens + dst_hyphens >= 2 {
            Err(Error::MultipleStdin)
        } else {
            Ok(())
        }
    }

    fn validate(&self) -> Result<(), Error> {
        self.validate_path()
    }

    pub fn to_quotients(&self) -> Result<(TableQuotient, TableQuotient)> {
        let _ = self.validate()?;

        let source_columns = self.source_columns.parse::<Columns>()
            .wrap_err("Failed to parse field names and types of source table")?;
        let target_columns = self.target_columns.parse::<Columns>()
            .wrap_err("Failed to parse field names and types of target table")?;

        let (source_mapping, target_mapping) = crate::expr::parse_equal(&self.predicate, &source_columns, &target_columns)
            .wrap_err("Failed to parse the value of `--predicate`")?;

        trace!("source columns: {:?}", source_columns);
        trace!("source mapping: {:?}", source_mapping);
        trace!("target columns: {:?}", target_columns);
        trace!("target mapping: {:?}", target_mapping);

        let source_table = TableBuilder::new()
            .columns(source_columns)
            .ignore_length_mismatch(self.ignore_length_mismatch)
            .ignore_parse_error(self.ignore_parse_error)
            .from_path(&self.source_file)
            .wrap_err("Failed to read source table")?;

        let target_table = TableBuilder::new()
            .columns(target_columns)
            .ignore_length_mismatch(self.ignore_length_mismatch)
            .from_path(&self.target_file)
            .wrap_err("Failed to read source table")?;

        Ok((TableQuotient::new(&source_table, &source_mapping.into()),
            TableQuotient::new(&target_table, &target_mapping.into())))
    }
}
