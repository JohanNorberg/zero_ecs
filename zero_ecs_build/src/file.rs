use std::{fs, path::Path, process::Command};

pub fn write_token_stream_to_file(out_dir: &str, file_name: &str, code: &str) -> String {
    let dest_path = Path::new(&out_dir).join(file_name);
    fs::write(&dest_path, code.to_string())
        .expect(format!("failed to write to file: {}", file_name).as_str());
    format_file(&dest_path.to_str().unwrap());
    format!("/{}", file_name)
}
pub fn format_file(file_name: &str) {
    Command::new("rustfmt")
        .arg(file_name)
        .output()
        .expect("failed to execute rustfmt");
}
