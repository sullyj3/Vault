
use std::fs::File;
use std::io;
use std::fmt;
use std::io::Write;
use std::io::prelude::*;
use std::process::exit;

use indoc::indoc;

// #[macro_use]
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

#[derive(Debug)]
enum ColType {
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
               });
        Ok(())
    }
}


#[derive(Debug, PartialEq)]
enum Value {
    IntType(i32),
    StringType(String)
}

#[derive(Debug)]
struct Column<'a> {
    colname: &'a str,
    coltype: ColType,
}

impl fmt::Display for Column<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}",
               self.colname,
               self.coltype
        );
        Ok(())
    }
}


type Row = Vec<Value>;

type TableSchema<'a> = Vec<Column<'a>>;

fn temp_table() -> TableSchema<'static> {
    vec![ Column { colname: "id",       coltype: ColType::IntType},
          Column { colname: "username", coltype: ColType::StringType},
          Column { colname: "email",    coltype: ColType::StringType} ]
}

#[derive(Debug, PartialEq)]
struct RowParseError;

// s will be a comma separated list of values
fn parse_row(s: &str, schema: TableSchema) -> Result<Row, RowParseError> {
    let vals: Vec<&str> = s.split(',').collect();
    if vals.len() != schema.len() {
        return Err(RowParseError);
    }

    let mut row = Vec::with_capacity(schema.len());

    for (col, val_str) in schema.iter().zip(vals.iter()) {
        let val = match col.coltype {
            ColType::IntType => {
                let i = val_str.parse::<i32>().map_err(|_| RowParseError)?;
                Value::IntType(i)
            }
            ColType::StringType => Value::StringType(val_str.to_string())
        };

        row.push(val);
    }

    Ok( row )
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

fn parse_row2(row: &str) -> IResult<&str, Row> {
    separated_nonempty_list(tag(","), parse_val)(row)
}

enum MetaCommandError {
    UnrecognisedCommand,
    CommandSyntaxError,
}
use crate::MetaCommandError::*;

#[derive(Debug)]
enum StatementPrepareError {
    UnrecognisedStatement
}
//use StatementPrepareError::*;

#[derive(Debug)]
enum Statement {
    Insert(Row),
    Select
}
use Statement::*;

fn show_schema(schema: &TableSchema) -> String {
    let mut result = "| ".to_string();
    for col in schema {
        result += &col.to_string();
        result += " | ";
    }
    result
}

fn main() -> std::io::Result<()> {
    let schema = temp_table();

    println!("Welcome to Vault!");
    println!("\nUsing the following schema:\n{}", show_schema(&schema) );
    shell(schema);

    Ok(())
}

fn prompt(s: &str) {
    print!("{}", s);
    io::stdout().flush().unwrap();
}

fn shell(_schema: TableSchema) {

    prompt("vault> ");
    for line in io::stdin().lock().lines().map( |l| l.unwrap() ) {

        if line.trim().len() == 0 {
            continue;
        } else if line.starts_with(":") {
            let mut chars = line.chars();
            chars.next();
            let command = chars.as_str();
            match handle_meta_command(&command) {
                Ok(()) => {}
                Err(UnrecognisedCommand) => {
                    println!("Unrecognised command: {}", line);
                    usage();
                }
                Err(CommandSyntaxError) => {
                    println!("invalid syntax: {}", line);
                    usage();
                }
            }
        } else {
            match handle_statement(&line) {
                Ok(()) => {}
                Err(e) => println!("Error handling statement, {:?}", e),
            }
        }
        prompt("vault> ");
    }
}

fn handle_statement(stat: &str) -> Result<(), StatementPrepareError> {
    let statement = parse_statement(stat)?;

    println!("{:?}", statement);
    execute_statement(statement);
    Ok(())

}

fn parse_insert(insert: &str) -> IResult<&str, Statement> {
    let (i,_) = tag_no_case("INSERT")(insert)?;
    let (i,_) = space1(i)?;
    let (i,row) = delimited(tag("("), parse_row2, tag(")"))(i)?;
    let statement = Insert(row);
    Ok((i, statement))
}

fn parse_select(select: &str) -> IResult<&str, Statement> {
    let (i,_) = tag_no_case("SELECT")(select)?;
    Ok((i, Select))
}

// TODO: proper errors
fn parse_statement(statement: &str) -> Result<Statement, StatementPrepareError> {
    alt( ( parse_insert, parse_select ) )(statement)
        .map( |(_, s)| s)
        .map_err( |_| StatementPrepareError::UnrecognisedStatement )
    
}

fn execute_statement(s: Statement) {
    match s {
        Insert(_) => println!("insert not implemented"),
        Select => println!("select not implemented"),
    }
}

fn handle_meta_command(cmd: &str) -> Result<(), MetaCommandError> {
    let words: Vec<&str> = cmd.split_whitespace().collect();
    let words_slice: &[&str] = words.as_ref();

    match words[0] {
        "exit" => {
            exit(0);
        }
        "help" => {
            usage();
        }
        "open" => {
            if let [_open, fname] = words_slice {
                if let Ok(_file) = File::open(fname) {
                    println!("{} exists!", fname);
                } else {
                    println!("{} does not exist.", fname);
                }
            } else {
                return Err(CommandSyntaxError)
            }
        }
        "create" => {
            if let [_create, fname] = words_slice {
                match File::create(fname) {
                    Ok(_file) => println!("{} created!", fname),
                    Err(e) => println!("creating {} failed. ({})", fname, e),
                }
            } else {
                return Err(CommandSyntaxError)
            }

        }
        _ => {
            return Err(UnrecognisedCommand)
        }
    }
    Ok(())
}

fn usage() {
    let usage = indoc!("
        Available commands:

            :open FILENAME
            :create FILENAME
            :help
            :exit
        ");
    println!("{}", usage)
}

#[test]
fn test_parse_row_basic() {
    let schema = temp_table();
    let data = "0,james,sullyj3@gmail.com";
    let result = parse_row(data, schema);
    let expected = Ok(vec![Value::IntType(0),
                           Value::StringType("james".to_string()),
                           Value::StringType("sullyj3@gmail.com".to_string())] );
    assert_eq!(result, expected);
}
