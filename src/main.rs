#![feature(ptr_metadata, new_uninit)]

use compiler::Compiler;


mod compiler;
mod scope;
mod gc;
mod value;
pub mod vm;

fn main() {
    let mut compiler = Compiler::new();

    compiler.compile_script(r#"
        function foo() {   
            return bar();
        }

        function bar() {
            return 0.0;
        }

        console.log(foo());
    "#).unwrap();
}
