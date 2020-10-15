use crate::Tag;
use thiserror::Error;

#[derive(Error,Debug)]
pub enum ParseError {
    #[error("type parse error")]
    InvalidType(#[from] strum::ParseError),
    #[error("invalid field specifier")]
    InvalidFieldSpec,
}

#[derive(Debug,Clone)]
pub struct Columns {
    columns: Vec<(String, Tag)>,
}

impl Columns {
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, (String, Tag)> {
        self.columns.iter()
    }

    pub fn get_by_name<T: AsRef<str>>(&self, s: T) -> Option<(usize, Tag)> {
        let pos = self.columns.iter()
            .position(|(name, _)| name == s.as_ref())?;
        Some((pos, self.columns[pos].1))
    }

    pub fn get_by_index(&self, index: usize) -> Option<(&str, Tag)> {
        self.columns.get(index)
            .map(|(name, tag)| (name.as_ref(), *tag))
    }
}

impl std::str::FromStr for Columns {
    type Err = self::ParseError;

    fn from_str(s: &str) -> Result<Columns, Self::Err> {
        parse_columns(s)?
            .into_iter()
            .map(|(name, tag)| {
                match tag.parse::<Tag>() {
                    Ok(tag) => Ok((name, tag)),
                    Err(e) => Err(e.into()),
                }
            })
            .collect::<Result<Vec<(String, Tag)>, Self::Err>>()
            .map(|col| Columns { columns: col })
    }
}

fn parse_name_type(s: &str) -> Result<(String, String), ParseError> {
    let fields: Vec<&str> = s.split(':')
        .map(|s| s.trim())
        .collect();

    if !(fields.len() == 2 || fields.len() == 1) {
        return Err(ParseError::InvalidFieldSpec);
    }

    if fields.len() == 1 {
        Ok((fields[0].to_owned(), "String".to_owned()))
    } else if fields.len() == 2 {
        Ok((fields[0].to_owned(), fields[1].to_owned()))
    } else {
        unreachable!()
    }
}

fn parse_columns<'a>(s: &'a str) -> Result<Vec<(String, String)>, ParseError> {
    let s = s.trim();
    if s.len() == 0 {
        Ok(Vec::new())
    } else {
        s.split(',')
            .map(|s| s.trim())
            .map(|s| parse_name_type(s))
            .collect()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn get() {
        use super::Columns;
        use crate::Tag;
        
        let cols: Columns = "key1:String, key2:Decimal".parse().unwrap();

        assert_eq!(cols.get_by_name("key1"), Some((0, Tag::String)));
        assert_eq!(cols.get_by_name("key2"), Some((1, Tag::Decimal)));

        assert_eq!(cols.get_by_index(0), Some(("key1", Tag::String)));
        assert_eq!(cols.get_by_index(1), Some(("key2", Tag::Decimal)));

        let cols = "".parse::<Columns>().unwrap();
        assert_eq!(cols.iter().next(), None);
    }

    #[test]
    fn parse_columns() {
        assert_eq!(super::parse_columns("key1:string,key2:decimal"),
        vec![("key1".to_owned(), "string".to_owned()),
             ("key2".to_owned(), "decimal".to_owned())]
        );
        assert_eq!(super::parse_columns("key1: string,  key2: decimal  "),
        vec![("key1".to_owned(), "string".to_owned()),
             ("key2".to_owned(), "decimal".to_owned())]
        );
        assert_eq!(super::parse_columns("key1,,key3"),
        vec![("key1".to_owned(), "String".to_owned()),
             (String::new(), "String".to_owned()),
             ("key3".to_owned(), "String".to_owned())]);
        assert_eq!(super::parse_columns(",,,,"),
        vec![(String::new(), "String".to_owned()); 5]);
    }
}
