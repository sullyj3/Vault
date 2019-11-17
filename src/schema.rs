use std::fmt;

#[derive(Debug)]
pub enum ColType {
    IntType,
    StringType,
}

impl fmt::Display for ColType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
               "{}",
               match self {
                   ColType::IntType => "Int",
                   ColType::StringType => "String",
               }).unwrap();
        Ok(())
    }
}


#[derive(Debug, PartialEq)]
pub enum Value {
    IntType(i32),
    StringType(String)
}

#[derive(Debug)]
pub struct Column<'a> {
    pub colname: &'a str,
    pub coltype: ColType,
}

impl fmt::Display for Column<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}",
               self.colname,
               self.coltype
        ).unwrap();
        Ok(())
    }
}


pub type Row = Vec<Value>;

pub type TableSchema<'a> = Vec<Column<'a>>;
