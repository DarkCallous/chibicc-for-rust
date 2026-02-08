use std::{process::Command, fs};

pub fn compile_and_run(input: &str) -> i32 {
    let output = Command::new("cargo")
        .args(&["run", "--", input])
        .output()
        .unwrap();
    let asm = String::from_utf8(output.stdout).unwrap();
    fs::write("tmp.s", asm).unwrap();
    
    // Use clang (LLVM's C compiler) to assemble and link
    Command::new("clang")
        .args(&["-o", "D:\\tmp.exe", "tmp.s"])
        .status()
        .unwrap();
    
    let result = Command::new("D:\\tmp.exe").status().unwrap();
    result.code().unwrap()
}
 
#[test]
fn test_literals() {
    assert_eq!(compile_and_run("return 0;"), 0);
    assert_eq!(compile_and_run("return 42;"), 42);
    assert_eq!(compile_and_run("return 255;"), 255);
}

#[test]
fn test_addition() {
    assert_eq!(compile_and_run("return 5+3;"), 8);
    assert_eq!(compile_and_run("return 1+2+3;"), 6);
}

#[test]
fn test_subtraction() {
    assert_eq!(compile_and_run("return 10-3;"), 7);
    assert_eq!(compile_and_run("return 5-2-1;"), 2);
}

#[test]
fn test_multiplication() {
    assert_eq!(compile_and_run("return 2*3;"), 6);
    assert_eq!(compile_and_run("return 5+6*7;"), 47);  // Your example!
}

#[test]
fn test_division() {
    assert_eq!(compile_and_run("return 8/2;"), 4);
    assert_eq!(compile_and_run("return 9/2;"), 4);  // Integer division
}

#[test]
fn test_complex_expressions() {
    assert_eq!(compile_and_run("return 5*(9-6);"), 15);
    assert_eq!(compile_and_run("return (3+5)/2;"), 4);
}

#[test]
fn test_unary_plus() {
    assert_eq!(compile_and_run("return +5;"), 5);
    assert_eq!(compile_and_run("return +42;"), 42);
    assert_eq!(compile_and_run("return 0+ +10;"), 10);  // 0 + (+10)
}

#[test]
fn test_unary_minus() {   
    assert_eq!(compile_and_run("return 10-5;"), 5);
    assert_eq!(compile_and_run("return 5+ -3;"), 2);    // 5 + (-3)
}

#[test]
fn test_unary_in_expressions() {
    assert_eq!(compile_and_run("return -5+10;"), 5);    // (-5) + 10
    assert_eq!(compile_and_run("return 10+ -5;"), 5);   // 10 + (-5)
}

#[test]
fn test_precedence_with_unary() {
    assert_eq!(compile_and_run("return -5*2+10;"), 0);   // ((-5)*2) + 10 = -10 + 10 = 0
}

#[test]
fn test_equality_precedence_with_unary() {
    assert_eq!(compile_and_run("return -5*2+10 != 0;"), 0);   // ((-5)*2) + 10 = -10 + 10 = 0
}