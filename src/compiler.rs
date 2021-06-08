use std::convert::TryInto;

use swc_common::BytePos;
use swc_ecmascript::{ast::{Decl, Function, Ident, Script, Stmt, FnDecl}, parser::{self, Parser, StringInput, Syntax}, visit::{self, Node, Visit, VisitWith}};

#[derive(Debug)]
pub struct Error(String);

impl From<parser::error::Error> for Error {
    fn from(e: parser::error::Error) -> Self {
        Error(e.into_kind().msg().into_owned())
    }
}

pub struct Compiler {
    
}

impl Compiler {
    pub fn new() -> Self {
        Self {

        }
    }

    fn function(&mut self, func: &Function) {

    }

    fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Block(_) => todo!(),
            Stmt::Empty(_) => todo!(),
            Stmt::Debugger(_) => todo!(),
            Stmt::With(_) => todo!(),
            Stmt::Return(_) => todo!(),
            Stmt::Labeled(_) => todo!(),
            Stmt::Break(_) => todo!(),
            Stmt::Continue(_) => todo!(),
            Stmt::If(_) => todo!(),
            Stmt::Switch(_) => todo!(),
            Stmt::Throw(_) => todo!(),
            Stmt::Try(_) => todo!(),
            Stmt::While(_) => todo!(),
            Stmt::DoWhile(_) => todo!(),
            Stmt::For(_) => todo!(),
            Stmt::ForIn(_) => todo!(),
            Stmt::ForOf(_) => todo!(),
            Stmt::Decl(decl) => match decl {
                Decl::Class(_) => todo!(),
                Decl::Fn(func) => {
                    println!("{:?}", func.ident);
                }
                Decl::Var(_) => todo!(),
                Decl::TsInterface(_) => todo!(),
                Decl::TsTypeAlias(_) => todo!(),
                Decl::TsEnum(_) => todo!(),
                Decl::TsModule(_) => todo!(),
            }
            Stmt::Expr(_) => todo!(),
        }
    }

    fn compile_stmts(&mut self, stmts: &[Stmt]) {
        // println!("{:#?}", stmts);

        for stmt in stmts {
            self.stmt(stmt);
        }
    }

    pub fn compile_script(&mut self, code: &str) -> Result<(), Error> {
        let mut parser = Parser::new(
            Syntax::Es(Default::default()),
            StringInput::new(code, BytePos(0), BytePos(code.len().try_into().unwrap())),
            None,
        );

        let script = parser.parse_script()?;

        self.compile_stmts(&script.body);

        Ok(())
    }
}

