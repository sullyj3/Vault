mod schema;
mod statement;

use std::fs::File;
use std::io;
use std::io::Write;
use std::io::prelude::*;
use std::process::exit;
use indoc::indoc;

use statement::{
    Statement::*,
    StatementPrepareError,
    parse_statement,
};

use schema::{
    ColType,
    Column,
    TableSchema
};

enum MetaCommandError {
    UnrecognisedCommand,
    CommandSyntaxError,
}
use crate::MetaCommandError::*;

fn main() -> std::io::Result<()> {
    let schema = temp_table();

    println!("Welcome to Vault!");
    println!("\nUsing the following schema:\n{}", show_schema(&schema) );
    shell(schema);

    Ok(())
}

fn temp_table() -> TableSchema<'static> {
    vec![ Column { colname: "id",       coltype: ColType::IntType},
          Column { colname: "username", coltype: ColType::StringType},
          Column { colname: "email",    coltype: ColType::StringType} ]
}

fn show_schema(schema: &TableSchema) -> String {
    let mut result = "|".to_string();
    for col in schema {
        result += &format!(" {} |", col.to_string());
    }
    result
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

fn prompt(s: &str) {
    print!("{}", s);
    io::stdout().flush().unwrap();
}

fn handle_meta_command(cmd: &str) -> Result<(), MetaCommandError> {
    let words: Vec<&str> = cmd.split_whitespace().collect();
    let words_slice: &[&str] = words.as_ref();

    match words[0] {
        "exit" => exit(0),
        "help" =>  usage(),
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

fn handle_statement(stat: &str) -> Result<(), StatementPrepareError> {
    let statement = parse_statement(stat)?;

    println!("{:?}", statement);
    match statement {
        Insert(_) => println!("insert not implemented"),
        Select => println!("select not implemented"),
    }
    Ok(())

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
