use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use csv::StringRecord;
use chrono::NaiveDate;
use thiserror::Error;
use log::warn;
use crate::{Tag, Value};
use crate::expr::Term;
use crate::columns::Columns;

#[derive(Debug,Clone,PartialEq,Eq,Hash,Ord,PartialOrd)]
pub struct Record(Vec<Value>);

impl From<Vec<Value>> for Record {
    fn from(vec: Vec<Value>) -> Record {
        Record(vec)
    }
}

impl Deref for Record {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Record {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut iter = self.0.iter();
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
            for field in iter {
                write!(f, ",{}", field)?;
            }
        }
        Ok(())
    }
}

#[derive(Error,Debug)]
pub enum ParseError {
    #[error("Field value is not a decimal")]
    Decimal(#[from] rust_decimal::Error),
    #[error("Field value is not a date")]
    Date,
    #[error("Lengths of specified columns and a record does not match: columns: {columns}, record: {record}")]
    DifferentLength {
        columns: usize,
        record: usize,
    },
}

#[derive(Debug)]
pub struct RecordParserBuilder<LenMis> {
    ignore_length_mismatch: LenMis,
}

impl RecordParserBuilder<()> {
    pub fn new() -> Self {
        RecordParserBuilder {
            ignore_length_mismatch: (),
        }
    }
}

impl RecordParserBuilder<bool> {
    pub fn from_columns<'a>(self, columns: &'a Columns) -> RecordParser<'a> {
        RecordParser {
            columns: columns,
            ignore_length_mismatch: self.ignore_length_mismatch,
        }
    }
}

impl<LenMis> RecordParserBuilder<LenMis> {
    pub fn ignore_length_mismatch(self, flag: bool) -> RecordParserBuilder<bool> {
        RecordParserBuilder {
            ignore_length_mismatch: flag,
        }
    }
}

#[derive(Debug,Clone)]
pub struct RecordParser<'a> {
    columns: &'a Columns,
    ignore_length_mismatch: bool,
}

impl<'a> RecordParser<'a> {
    pub fn parse(&self, record: &StringRecord) -> Result<Option<Record>, ParseError> {
        if self.columns.len() != record.len() {
            if self.ignore_length_mismatch {
                warn!("Length mismatch: Ignored a record: {:?}", record);
                return Ok(None);
            } else {
                return Err(ParseError::DifferentLength {
                    columns: self.columns.len(),
                    record: record.len(),
                });
            }
        }

        let fields = record.iter()
            .zip(self.columns.iter().map(|(_, tag)| *tag))
            .map(|(field, tag)| {
                match tag {
                    Tag::String => Ok(Value::String(field.to_owned())),
                    Tag::Decimal => field.parse()
                        .map(|d| Value::Decimal(d))
                        .map_err(|e| e.into()),
                    Tag::Date => {
                        if let Ok(date) = NaiveDate::parse_from_str(field, "%Y-%m-%d") {
                            Ok(Value::Date(date))
                        } else if let Ok(date) = NaiveDate::parse_from_str(field, "%Y/%m/%d") {
                            Ok(Value::Date(date))
                        } else if let Ok(date) = NaiveDate::parse_from_str(field, "%Y年%m月%d日") {
                            Ok(Value::Date(date))
                        } else {
                            Err(ParseError::Date)
                        }
                    },
                }
            })
            .collect::<Result<Vec<Value>, ParseError>>();

        fields.map(|f| Some(Record(f)))
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct Mapping(Vec<Term>);

impl From<Vec<Term>> for Mapping {
    fn from(terms: Vec<Term>) -> Self {
        Self(terms)
    }
}

impl Mapping {
    pub fn apply(&self, fields: &[Value]) -> Vec<Value> {
        self.0.iter()
            .map(|term| term.eval(fields))
            .collect()
    }
}

impl crate::quotient::Projection for Mapping {
    type Domain = Record;
    type Target = Record;

    fn project(&self, x: &Self::Domain) -> Self::Target {
        Record(self.apply(&x.0))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Value;
    use crate::expr::Term;

    #[test]
    fn mapping_apply() {
        let mapping: Mapping = vec![Term::Val(1), Term::Neg(0)].into();
        let record = vec![Value::Decimal(10), Value::String("hello".to_owned())];
        assert_eq!(mapping.apply(&record),
            vec![Value::String("hello".to_owned()), Value::Decimal(-10)]);
    }
}
