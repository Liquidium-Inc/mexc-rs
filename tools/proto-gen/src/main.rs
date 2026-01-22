use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

const WS_PROTO_SUB_NAME: &str = "websocket-proto";

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let repo_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .ok_or("Failed to find repo root from tools/proto-gen")?;

    let ws_proto_dir = repo_root.join(WS_PROTO_SUB_NAME);

    let protoc = protobuf_src::protoc();
    env::set_var("PROTOC", protoc.as_os_str());

    if !ws_proto_dir.exists() {
        run_command_checked(repo_root, "git", &["submodule", "update", "--init", WS_PROTO_SUB_NAME])?;
    }

    if !ws_proto_dir.exists() {
        return Err(format!(
            "Directory {} does not exist after initializing submodule",
            ws_proto_dir.display()
        )
        .into());
    }

    let mut proto_files =
        collect_proto_files(&ws_proto_dir).map_err(|err| format!("Failed to read proto files: {err}"))?;
    if proto_files.is_empty() {
        return Err(format!("No .proto files found in {}", ws_proto_dir.display()).into());
    }
    proto_files.sort();

    let out_dir = repo_root.join("target/proto-gen");
    fs::create_dir_all(&out_dir)?;

    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .out_dir(&out_dir)
        .compile_protos(&proto_files, &[ws_proto_dir])?;

    let generated = out_dir.join("_.rs");
    if !generated.exists() {
        return Err(format!("Expected generated file at {}", generated.display()).into());
    }

    let dest_dir = repo_root.join("src/proto");
    fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join("mod.rs");
    if dest.exists() {
        fs::remove_file(&dest)?;
    }
    fs::rename(&generated, &dest)?;

    println!("Wrote {}", dest.display());
    Ok(())
}

fn collect_proto_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let next = collect_proto_files(&path)?;
            out.extend(next);
        } else if path.extension().is_some_and(|ext| ext == "proto") {
            out.push(path);
        }
    }
    Ok(out)
}

fn run_command_checked(
    repo_root: &Path,
    cmd: &str,
    args: &[&str],
) -> Result<(), Box<dyn Error>> {
    let status = Command::new(cmd)
        .args(args)
        .current_dir(repo_root)
        .status()?;

    if !status.success() {
        return Err(format!("Command failed: {cmd:?} {args:?}").into());
    }

    Ok(())
}
