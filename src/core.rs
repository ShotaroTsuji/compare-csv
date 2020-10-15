use std::collections::HashSet;
use eyre::Result;
use crate::table::Table;
use crate::columns::Columns;
use crate::record::{Record, Mapping};
use crate::quotient::Quotient;
use crate::app::App;

pub struct TableQuotient {
    columns: Columns,
    quotient: Quotient<Record, Record, Mapping>,
}

impl TableQuotient {
    pub fn new(table: &Table, proj: &Mapping) -> TableQuotient {
        let mut quot = Quotient::with_projection(proj.clone());

        for record in table.iter() {
            quot.push(record.clone());
        }

        TableQuotient {
            columns: table.columns().clone(),
            quotient: quot,
        }
    }

    pub fn columns(&self) -> &Columns {
        &self.columns
    }

    pub fn quotient(&self) -> &Quotient<Record, Record, Mapping> {
        &self.quotient
    }

    pub fn difference<'a>(&'a self, other: &'a Self) -> crate::quotient::Difference<'a, Record, Record, Mapping> {
        self.quotient.difference(&other.quotient)
    }
}

pub struct TableComparator {
    source: TableQuotient,
    target: TableQuotient,
    empty_vec: Vec<Record>,
}

impl TableComparator {
    pub fn from_app(app: &App) -> Result<Self> {
        let (source, target) = app.to_quotients()?;

        Ok(Self {
            source: source,
            target: target,
            empty_vec: Vec::new(),
        })
    }

    pub fn different_points<'a>(&'a self) -> Vec<&'a Record> {
        let mut points = HashSet::new();

        for pt in self.source.difference(&self.target) {
            points.insert(pt);
        }
        for pt in self.target.difference(&self.source) {
            points.insert(pt);
        }

        let mut points: Vec<&Record> = points.drain().collect();
        points.sort();

        points
    }

    pub fn get_records(&self, pt: &Record) -> (&[Record], &[Record]) {
        let src_class = match self.source.quotient().get(pt) {
            Some(class) => class,
            None => &self.empty_vec,
        };
        let tgt_class = match self.target.quotient().get(pt) {
            Some(class) => class,
            None => &self.empty_vec,
        };
        (src_class, tgt_class)
    }
}
