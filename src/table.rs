use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::io::Read;
use std::fs::File;
use csv::ReaderBuilder;
use thiserror::Error;
use log::warn;
use crate::columns::Columns;
use crate::record::{RecordParserBuilder, RecordParser, Record};

#[derive(Error,Debug)]
pub enum Error {
    #[error("Failed to parse a record: line {line}")]
    ParseRecord {
        source: crate::record::ParseError,
        line: usize,
    },
    #[error("Failed to open a file")]
    FileOpen(#[from] std::io::Error),
    #[error("Failed to read a CSV record")]
    Csv(#[from] csv::Error),
}

pub fn read_csv_records<'a, R: Read>(rdr: R, records: &mut Vec<Record>, parser: &RecordParser<'a>, ignore_parse_error: bool) -> Result<(), Error> {
    let rdr = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(rdr);

    for (index, record) in rdr.into_records().enumerate() {
        let record = record.map_err(|e| Error::Csv(e))?;
        let record = match parser.parse(&record) {
            Ok(record) => record,
            Err(e) => {
                if ignore_parse_error {
                    warn!("Ignored a parse error: {} at line {}; {:?}", e, index+1, record);
                    continue;
                } else {
                    return Err(Error::ParseRecord {
                        source: e,
                        line: index+1,
                    });
                }
            },
        };
        if let Some(record) = record {
            records.push(record);
        }
    }

    Ok(())
}

#[derive(Debug,Default)]
pub struct TableBuilder {
    columns: Option<Columns>,
    ignore_length_mismatch: bool,
    ignore_parse_error: bool,
}

impl TableBuilder {
    pub fn new() -> Self {
        TableBuilder::default()
    }

    pub fn columns(self, columns: Columns) -> Self {
        TableBuilder {
            columns: Some(columns),
            ..self
        }
    }

    pub fn ignore_length_mismatch(self, flag: bool) -> Self {
        TableBuilder {
            ignore_length_mismatch: flag,
            ..self
        }
    }

    pub fn ignore_parse_error(self, flag: bool) -> Self {
        TableBuilder {
            ignore_parse_error: flag,
            ..self
        }
    }

    pub fn from_path<P: AsRef<Path>>(self, input: &[P]) -> Result<Table, Error> {
        let hyphen = PathBuf::from_str("-").unwrap();

        let columns = self.columns.unwrap();

        let parser = RecordParserBuilder::new()
            .ignore_length_mismatch(self.ignore_length_mismatch)
            .from_columns(&columns);

        let mut data = Vec::new();

        for path in input.iter() {
            let path = path.as_ref();

            if path == &hyphen {
                read_csv_records(std::io::stdin(), &mut data, &parser, self.ignore_parse_error)?;
            } else {
                let f = File::open(&path)
                    .map_err(|e| Error::FileOpen(e))?;
                read_csv_records(f, &mut data, &parser, self.ignore_parse_error)?;
            }
        }

        Ok(Table {
            columns: columns,
            data: data,
        })
    }
}

#[derive(Debug,Clone)]
pub struct Table {
    columns: Columns,
    data: Vec<Record>,
}

impl Table {
    pub fn columns(&self) -> &Columns {
        &self.columns
    }

    pub fn inner(&self) -> &[Record] {
        &self.data
    }

    pub fn iter(&self) -> impl Iterator<Item=&Record> {
        self.data.iter()
    }
}
