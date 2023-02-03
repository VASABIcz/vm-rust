extern crate rust_vm;

use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::io::{BufRead, Write};
use std::process::exit;

use rust_vm::ast;
use rust_vm::codegen::{bytecodeGen, complexBytecodeGen};
use rust_vm::lexer::{Token, tokenizeSource};
use rust_vm::parser::{Operation, parse, parseOne, parseTokens, parsingUnits, TokenProvider};
use rust_vm::parser::ParsingUnitSearchType::{Ahead, Around};
use rust_vm::vm::{bootStrapVM, DataType, evaluateBytecode, OpCode, run, SeekableOpcodes, StackFrame, Value, VirtualMachine};
use rust_vm::vm::RawOpCode::PushInt;

fn readInput() -> String {
    print!(">>> ");
    io::stdout().flush();
    let stdin = io::stdin();
    let mut buf = String::new();
    stdin.lock().read_line(&mut buf);

    if buf == "EXIT\n" {
        exit(0);
    }

    return buf;
}

fn handleError(err: Box<dyn Error>) {
    eprintln!("ERROR: {}", err)
}

fn main() {
    let mut vm = bootStrapVM();
    let mut localTypes = vec![];
    let mut functionReturns = HashMap::new();
    let mut mainLocals = HashMap::new();
    let mut localValues = vec![];
    let mut lastLocalSize: usize = 0;
    let mut opcodeIndex: usize = 0;
    let mut opcodes = vec![];
    let parsingUnits = parsingUnits();

    println!("VIPL-repl");
    println!("(vasuf insejn programing language)");
    println!("to exit type ^C or EXIT");

    loop {
        let str = readInput();

        let mut tokens = match tokenizeSource(&str) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("tokenizer");
                handleError(e);
                continue
            }
        };
        if tokens.is_empty() {
            continue
        }

        let mut tokenProvider = TokenProvider { tokens, index: 0 };
        let mut first = match parseOne(&mut tokenProvider, Ahead, &parsingUnits, None) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("first parser");
                handleError(e);
                continue
            }
        };
        let mut isPrevUsed = false;

        // println!("first {:?}", &first);

        let res = if !tokenProvider.isDone() {
            match parse(&mut tokenProvider, Around, &parsingUnits, Some(first.clone()), &mut isPrevUsed) {
                Ok(mut v) => {
                    if !isPrevUsed {
                        v.push(first);
                    }
                    v
                },
                Err(e) => {
                    eprintln!("parser");
                    handleError(e);
                    continue
                }
            }
        } else {
            vec![first]
        };
        // println!("{:?}", &res);

        let bs = match complexBytecodeGen(res, &mut localTypes, &mut functionReturns, &mut mainLocals) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("bytecode");
                handleError(e);
                continue;
            }
        };
        if lastLocalSize < localTypes.len() {
            for x in &localTypes[lastLocalSize..localTypes.len()] {
                localValues.push(x.toDefaultValue())
            }
        }

        // println!("{:?}", &bs);

        for _ in 0..bs.len() {
            vm.opCodeCache.push(None)
        }

        opcodes.extend(bs);

        let mut stack = StackFrame {
            previous: None,
            localVariables: &mut localValues,
            name: None,
        };

        let mut opCodes = SeekableOpcodes {
            index: opcodeIndex as isize,
            opCodes: &opcodes,
            start: None,
            end: None,
        };

        run(&mut opCodes, &mut vm, &mut stack);
        opcodeIndex = opCodes.index as usize - 1;

        for val in &vm.stack {
            match val {
                Value::Num(v) => println!("{}", v),
                Value::Flo(v) => println!("{}", v),
                Value::Bol(v) => println!("{}", v),
                Value::Reference() => println!("object")
            }
        }
        vm.stack.clear();
    }
}