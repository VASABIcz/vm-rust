extern crate rust_vm;

use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use std::time::Instant;

use rust_vm::codegen::bytecodeGen2;
use rust_vm::fs::setupFs;
use rust_vm::lexer::tokenizeSource;
use rust_vm::parser::parseTokens;
use rust_vm::std::bootStrapVM;
use rust_vm::vm::evaluateBytecode2;

fn handleError(err: Box<dyn Error>) {
    eprintln!("ERROR: {err}");
    eprintln!("ERROR: {err:?}");
}

fn main() {
    let now = Instant::now();
    let sourceFile = std::env::args().nth(1).expect("expected source field");

    let src = std::fs::read_to_string(sourceFile).expect("failed to read source");

    let mut vm = bootStrapVM();
    // let mut localTypes = vec![];
    setupFs(&mut vm);
    let mut functionReturns = HashMap::new();

    for f in &vm.functions {
        functionReturns.insert(f.0.clone(), f.1.returnType.clone());
    }

    let tokens = match tokenizeSource(&src) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("tokenizer");
            handleError(e);
            return;
        }
    };

    if tokens.is_empty() {
        return;
    }

    // println!("tokens {:?}", &tokens);

    let ast = match parseTokens(tokens) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("parser");
            handleError(e);
            return;
        }
    };

    // println!("{:?}", &vm.functions.keys());

    let mut rets = HashMap::new();

    for f in &vm.functions {
        rets.insert(f.0.clone(), f.1.returnType.clone());
    }

    let bs = match bytecodeGen2(ast, &mut rets) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("codegen");
            handleError(e);
            return;
        }
    };

    // println!("{:?}", &bs.0);

    /*
    match checkBytecode(&mut SeekableOpcodes {
        index: 0,
        opCodes: &bs.0,
        start: None,
        end: None,
    }, &mut bs.1, &mut AbstractStack { stack: vec![] }, &mut vm, &mut HashSet::new()) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("bytecode check");
            handleError(e);
            return;
        }
    }

     */
    let e = now.elapsed();
    println!("compiled in: {e:.2?}");

    let a = Instant::now();

    evaluateBytecode2(bs.0, bs.1, &mut vm);

    let elapsed = a.elapsed();
    println!("finished in: {elapsed:.2?}");
}
