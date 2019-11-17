extern crate nom;

use nom::{
  IResult,
  branch::alt,
  bytes::complete::{tag, tag_no_case, is_not},
  character::complete::{space1, digit1},
  combinator::map,
  multi::separated_nonempty_list,
  sequence::delimited,
};

use crate::schema::{
    Row,
    Value,
};


#[derive(Debug)]
pub enum StatementPrepareError {
    UnrecognisedStatement
}

#[derive(Debug)]
pub enum Statement {
    Insert(Row),
    Select
}
use Statement::*;

// TODO: proper errors
pub fn parse_statement(statement: &str) -> Result<Statement, StatementPrepareError> {
    alt( ( parse_insert, parse_select ) )(statement)
        .map( |(_, s)| s)
        .map_err( |_| StatementPrepareError::UnrecognisedStatement )
}

fn parse_insert(insert: &str) -> IResult<&str, Statement> {
    let (i,_) = tag_no_case("INSERT")(insert)?;
    let (i,_) = space1(i)?;
    let (i,row) = delimited(tag("("), parse_row, tag(")"))(i)?;
    let statement = Insert(row);
    Ok((i, statement))
}

fn parse_select(select: &str) -> IResult<&str, Statement> {
    let (i,_) = tag_no_case("SELECT")(select)?;
    Ok((i, Select))
}


fn parse_val(i: &str) -> IResult<&str, Value> {
    alt(
        ( map(parse_int,       |int: i32| Value::IntType(int)),
          map(parse_string, |s: String| Value::StringType(s)) )
    )(i)
}

fn parse_int(i: &str) -> IResult<&str, i32> {
    let (i, ds) = digit1(i)?;
    let int = ds.parse::<i32>().unwrap();
    Ok((i, int))
}

fn parse_string(i: &str) -> IResult<&str, String> {
    map(delimited(tag("\""),
              is_not("\""),
              tag("\"")
    ), |s: &str| s.to_string() )(i)
}

fn parse_row(row: &str) -> IResult<&str, Row> {
    separated_nonempty_list(tag(","), parse_val)(row)
}
