// SPDX-License-Identifier: Unlicense
use std::{path::{Path, PathBuf}, io::Read};

use crate::{codegen, irgen, parser};
use anyhow::{anyhow, Result};

pub fn read_file(source: &Path) -> Result<String> {
    let mut buf = String::new();
    let mut f = std::fs::File::open(source)?;
    f.read_to_string(&mut buf)?;
    Ok(buf)
}

pub fn generate_object_from_string(name: &str, source: &str, out_dir: Option<PathBuf>) -> Result<PathBuf> {
    let (ast_arena, ast_root) = parser::parse(source)?;
    let (ir_arena, ir_root) = irgen::generate(ast_arena, ast_root)?;
    let context = inkwell::context::Context::create();
    let target_machine = codegen::get_host_target_machine()?;
    let codegen = codegen::CodeGen::new(ir_arena, &context, target_machine, name);
    codegen.generate(ir_root)?;
    let mut output = out_dir.unwrap_or(std::env::current_dir()?);
    output.push(name);
    output.set_extension("o");
    codegen.write_to_file(&output.as_path())?;
    Ok(output)
}

pub fn execute_linker(source: &Path) -> Result<PathBuf> {
    let cc = std::env::var("CC").unwrap_or("gcc".into());
    let ext = if cfg!(windows) { "exe" } else { "" };

    let mut output_path = PathBuf::from(source);
    output_path.set_extension(ext);
    
    let compiling = std::process::Command::new(cc)
        .args(vec![source.as_os_str() , std::ffi::OsStr::new("-o"), output_path.as_os_str()])
        .output()?;

    let stderr = String::from_utf8(compiling.stderr)?;
    let status = compiling
        .status
        .code()
        .ok_or(anyhow!("failed to execute the compiler"))?;
    if status != 0 {
        return Err(anyhow!(
            "compile failed with code {}\nstderr: {}",
            status,
            stderr
        ));
    }

    Ok(output_path)
}

pub fn compile(source: &Path) -> Result<PathBuf> {
    let src = read_file(source)?;
    let out_dir = PathBuf::from(source.parent().unwrap_or(&source));
    let mod_name = source.file_stem().and_then(|n| n.to_str()).unwrap_or("a");
    let obj = generate_object_from_string(mod_name, src.as_str(), Some(out_dir))?;
    let exe = execute_linker(obj.as_path())?;
    Ok(exe)
}

#[cfg(test)]
mod tests {
    use std::{env, fs::File, io::Write, path::Path, process::{Command, Output}};
    use anyhow::Result;
    use super::*;

    fn compile_and_run(name: &str, src: &str) -> Result<Output> {
        let test_dir = env::current_dir()?.join("test-data");
        let src_file = test_dir.join(format!("{name}.bonsai"));
        let mut f = File::options()
            .write(true)
            .create(true)
            .open(&src_file)?;

        f.write_all(src.as_bytes())?;
        let exe = compile(Path::new(&src_file))?;
        let output = Command::new(exe).output()?;
        Ok(output)
    }

    #[test]
    fn compiler_should_compile_basic_expression() -> Result<()> {
        let src = r#"
        6 * 7
        "#;

        let output = compile_and_run("basic_expression", src)?;
        let stdout = String::from_utf8(output.stdout)?;
        assert!(stdout.trim() == "result: 42");
        Ok(())
    }
}