use std::{fs::{self, remove_file}, process::Command};
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(0);

fn unique_base() -> String {
    let pid = std::process::id();
    let tid = format!("{:?}", std::thread::current().id());
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("chibicc_{pid}_{tid}_{n}")
}

pub fn compile_and_run(input: &str) -> i32 {
    let output = Command::new("cargo")
        .args(&["run", "--", input])
        .output()
        .unwrap();
    let asm = String::from_utf8(output.stdout).unwrap();
    let dir = std::env::temp_dir();
    let base = unique_base();
    let obj_path = dir.join(format!("{base}.s"));
    fs::write(obj_path.clone(), asm).unwrap();
    
    let exe_path = dir.join(format!("{base}.exe"));
    // Use clang (LLVM's C compiler) to assemble and link
    Command::new("clang")
        .args(&["-o", exe_path.to_str().unwrap(), obj_path.to_str().unwrap()])
        .status()
        .unwrap();
    
    let result = Command::new(exe_path.clone()).status().unwrap();
    let _ = remove_file(obj_path);
    let _ = remove_file(exe_path);
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