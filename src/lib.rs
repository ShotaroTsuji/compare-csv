use strum_macros::EnumString;
use chrono::NaiveDate;
use rust_decimal::prelude::*;

pub mod app;
pub mod core;
pub mod columns;
pub mod expr;
pub mod table;
pub mod record;
pub mod quotient;

#[derive(Debug,Clone,Copy,PartialEq,EnumString)]
pub enum Tag {
    String,
    Decimal,
    Date,
}

#[derive(Debug,Clone,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub enum Value {
    String(String),
    Decimal(Decimal),
    Date(NaiveDate),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Decimal(n) => write!(f, "{}", n),
            Value::Date(d) => write!(f, "{}", d),
        }
    }
}
