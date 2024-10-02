use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/quake_prefecture.proto"], &["src/"])?;
    Ok(())

}
