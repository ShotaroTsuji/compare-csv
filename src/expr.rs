use crate::{Value, Tag};
use crate::columns::Columns;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("an equals sign is not found")]
    EqualsSignNotFound,
    #[error("a closing parenthesis is missing")]
    MissingCloseParen,
    #[error("operation is not defined")]
    TypeError,
    #[error("Undefined field name: {0}")]
    UndefinedField(String),
}

pub fn parse_equal(s: &str, rcols: &Columns, lcols: &Columns) -> Result<(Vec<Term>, Vec<Term>), ParseError> {
    let pos = s.find('=')
        .ok_or(ParseError::EqualsSignNotFound)?;

    let rhs = s[..pos].trim();
    let lhs = s[pos+1..].trim();

    let rhs = parse_tuple_or_term(rhs, rcols)?;
    let lhs = parse_tuple_or_term(lhs, lcols)?;

    Ok((rhs, lhs))
}

fn parse_tuple_or_term(s: &str, cols: &Columns) -> Result<Vec<Term>, ParseError> {
    if let Some(tuple) = parse_tuple(s, cols)? {
        Ok(tuple.0)
    } else {
        Ok(vec![parse_term(s, cols)?])
    }
}

#[derive(Debug,Clone,PartialEq)]
pub struct Tuple(Vec<Term>);

fn parse_tuple(s: &str, cols: &Columns) -> Result<Option<Tuple>, ParseError> {
    let s = s.trim();

    if s.starts_with('(') {
        let end = s.find(')')
            .ok_or(ParseError::MissingCloseParen)?;
        Ok(Some(Tuple(parse_terms(&s[1..end], cols)?)))
    } else {
        Ok(None)
    }
}

fn parse_terms(s: &str, cols: &Columns) -> Result<Vec<Term>, ParseError> {
    let s = s.trim();

    if s.len() == 0 {
        Ok(Vec::new())
    } else {
        s.split(',')
            .map(|s| s.trim())
            .map(|s| parse_term(s, cols))
            .collect()
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum Term {
    Val(usize),
    Neg(usize),
}

impl Term {
    pub fn eval(&self, fields: &[Value]) -> Value {
        match *self {
            Term::Val(index) => fields.get(index).cloned().unwrap(),
            Term::Neg(index) => {
                match fields.get(index).cloned().unwrap() {
                    Value::Decimal(x) => Value::Decimal(-x),
                    Value::String(_) => panic!("Negative of String is undefined"),
                    Value::Date(_) => panic!("Negative of Date is undefined"),
                }
            },
        }
    }
}

fn parse_term(s: &str, columns: &Columns) -> Result<Term, ParseError> {
    let s = s.trim();

    if s.starts_with('-') {
        let name = s[1..].trim();
        let (index, tag) = columns.get_by_name(name)
            .ok_or(ParseError::UndefinedField(name.to_owned()))?;

        if tag != Tag::Decimal {
            return Err(ParseError::TypeError);
        }

        Ok(Term::Neg(index))
    } else {
        let (index, _) = columns.get_by_name(s)
            .ok_or(ParseError::UndefinedField(s.to_owned()))?;
        Ok(Term::Val(index))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn term() {
        let cols = "key1:Decimal, key2:Decimal".parse::<Columns>().unwrap();
        assert_eq!(parse_term("key1", &cols), Term::Val(0));
        assert_eq!(parse_term("-key2", &cols), Term::Neg(1));
    }

    #[test]
    fn tuple() {
        let cols = "x:Decimal, y:Decimal".parse::<Columns>().unwrap();
        assert_eq!(parse_tuple("(x, y)", &cols), Some(Tuple(vec![Term::Val(0), Term::Val(1)])));
    }

    #[test]
    fn tuple_neg() {
        let cols = "x:Decimal, y:Decimal".parse::<Columns>().unwrap();
        assert_eq!(parse_tuple("(x, -y)", &cols), Some(Tuple(vec![Term::Val(0), Term::Neg(1)])));
    }

    #[test]
    fn equal_term() {
        let rcols = "x: Decimal, y: Decimal".parse::<Columns>().unwrap();
        let lcols = "a: Decimal, b: Decimal".parse::<Columns>().unwrap();
        assert_eq!(parse_equal("x = -b", &rcols, &lcols),
            (vec![Term::Val(0)], vec![Term::Neg(1)]));
    }

    #[test]
    fn equal_tuple() {
        let rcols = "x: Decimal, y: Decimal".parse::<Columns>().unwrap();
        let lcols = "a: Decimal, b: Decimal".parse::<Columns>().unwrap();
        assert_eq!(parse_equal("(x, -y) = (b, a)", &rcols, &lcols),
            (vec![Term::Val(0), Term::Neg(1)], vec![Term::Val(1), Term::Val(0)]));
    }
}
