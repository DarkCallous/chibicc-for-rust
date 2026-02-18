use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(0);

fn unique_base() -> String {
    let pid = std::process::id();
    let tid = format!("{:?}", std::thread::current().id());
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    format!("chibicc_it_{pid}_{tid}_{n}")
}

fn compile_and_run(source: &str) -> Result<i32, String> {
    let dir = std::env::temp_dir();
    let base = unique_base();
    let src_path = dir.join(format!("{base}.c"));
    let asm_path = dir.join(format!("{base}.s"));
    let exe_path = dir.join(format!("{base}.exe"));

    fs::write(&src_path, source).map_err(|e| format!("write src failed: {e}"))?;

    let cc = env!("CARGO_BIN_EXE_chibicc-for-rust");
    let out = Command::new(cc)
        .arg(&src_path)
        .output()
        .map_err(|e| format!("invoke compiler failed: {e}"))?;

    if !out.status.success() {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        let _ = fs::remove_file(&src_path);
        return Err(format!(
            "compiler failed.\nstdout:\n{stdout}\nstderr:\n{stderr}"
        ));
    }

    fs::write(&asm_path, &out.stdout).map_err(|e| format!("write asm failed: {e}"))?;

    let clang = Command::new("clang")
        .args(["-o", exe_path.to_str().unwrap(), asm_path.to_str().unwrap()])
        .output()
        .map_err(|e| format!("invoke clang failed: {e}"))?;

    if !clang.status.success() {
        let stderr = String::from_utf8_lossy(&clang.stderr);
        let _ = fs::remove_file(&src_path);
        let _ = fs::remove_file(&asm_path);
        return Err(format!("clang failed:\n{stderr}"));
    }

    let run = Command::new(&exe_path)
        .status()
        .map_err(|e| format!("run exe failed: {e}"))?;

    let code = run
        .code()
        .ok_or_else(|| "process terminated by signal".to_string())?;

    let _ = fs::remove_file(&src_path);
    let _ = fs::remove_file(&asm_path);
    let _ = fs::remove_file(&exe_path);

    Ok(code)
}

fn run(source: &str) -> i32 {
    compile_and_run(source).unwrap()
}

#[test]
fn call_three_params_regression() {
    let code = r#"
        fma(a, b, c){
            return a*b+c;
        }
        main(){
            return fma(5, 6, 2);
        }
    "#;
    let rc = run(code);
    assert_eq!(rc, 32);
}

#[test]
fn forward_function_call_should_work() {
    let code = r#"
        main(){
            return foo(3);
        }
        foo(a){
            return a + 1;
        }
    "#;
    let rc = run(code);
    assert_eq!(rc, 4);
}

#[test]
fn fifth_argument_should_be_read_correctly() {
    let code = r#"
        foo(a, b, c, d, e){
            return e;
        }
        main(){
            return foo(1, 2, 3, 4, 5);
        }
    "#;
    let rc = run(code);
    assert_eq!(rc, 5);
}

#[test]
fn test_literals() {
    assert_eq!(run("main(){return 0;}"), 0);
    assert_eq!(run("main(){return 42;}"), 42);
    assert_eq!(run("main(){return 255;}"), 255);
}

#[test]
fn test_addition() {
    assert_eq!(run("main(){return 5+3;}"), 8);
    assert_eq!(run("main(){return 1+2+3;}"), 6);
}

#[test]
fn test_subtraction() {
    assert_eq!(run("main(){return 10-3;}"), 7);
    assert_eq!(run("main(){return 5-2-1;}"), 2);
}

#[test]
fn test_multiplication() {
    assert_eq!(run("main(){return 2*3;}"), 6);
    assert_eq!(run("main(){return 5+6*7;}"), 47); // Your example!
}

#[test]
fn test_division() {
    assert_eq!(run("main(){return 8/2;}"), 4);
    assert_eq!(run("main(){return 9/2;}"), 4); // Integer division
}

#[test]
fn test_complex_expressions() {
    assert_eq!(run("main(){return 5*(9-6);}"), 15);
    assert_eq!(run("main(){return (3+5)/2;}"), 4);
}

#[test]
fn test_unary_plus() {
    assert_eq!(run("main(){return +5;}"), 5);
    assert_eq!(run("main(){return +42;}"), 42);
    assert_eq!(run("main(){return 0+ +10;}"), 10); // 0 + (+10)
}

#[test]
fn test_unary_minus() {
    assert_eq!(run("main(){return 10-5;}"), 5);
    assert_eq!(run("main(){return 5+ -3;}"), 2); // 5 + (-3)
}

#[test]
fn test_unary_in_expressions() {
    assert_eq!(run("main(){return -5+10;}"), 5); // (-5) + 10
    assert_eq!(run("main(){return 10+ -5;}"), 5); // 10 + (-5)
}

#[test]
fn test_precedence_with_unary() {
    assert_eq!(run("main(){return -5*2+10;}"), 0); // ((-5)*2) + 10 = -10 + 10 = 0
}

#[test]
fn test_equality_precedence_with_unary() {
    assert_eq!(run("main(){return -5*2+10 != 0;}"), 0); // ((-5)*2) + 10 = -10 + 10 = 0
}

#[test]
fn test_fn_call_without_param() {
    assert_eq!(
        run(r#"
        fma(a, b, c){
            return a*b+c;
        }
        main(){
            return fma(5, 6, 2);
        }
        "#),
        32
    );
}

#[test]
fn test_addr_deref_roundtrip() {
    assert_eq!(run("main() { x=3; return *&x; }"), 3);
}

#[test]
fn test_multi_level_deref() {
    assert_eq!(run("main() { x=3; y=&x; z=&y; return **z; }"), 3);
}

#[test]
fn test_deref_with_addrof_plus_offset() {
    assert_eq!(run("main() { x=3; y=5; return *(&x-8); }"), 5);
}

#[test]
fn test_deref_with_addrof_minus_offset() {
    assert_eq!(run("main() { x=3; y=5; return *(&y+8); }"), 3);
}

#[test]
fn test_store_through_pointer() {
    assert_eq!(run("main() { x=3; y=&x; *y=5; return x; }"), 5);
}

#[test]
fn test_store_with_addrof_plus_offset() {
    assert_eq!(run("main() { x=3; y=5; *(&x-8)=7; return y; }"), 7);
}

#[test]
fn test_store_with_addrof_minus_offset() {
    assert_eq!(run("main() { x=3; y=5; *(&y+8)=7; return x; }"), 7);
}
