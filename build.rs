use std::{env, io, path};

fn main() -> io::Result<()> {
    let src = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = path::Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
    let mut options = built::Options::default();

    options.set_cfg(false);
    options.set_compiler(false);
    options.set_dependencies(false);
    options.set_env(false);
    options.set_features(false);

    options.set_git(true);
    options.set_ci(true);

    built::write_built_file_with_opts(&options, src.as_ref(), &dst)?;
    Ok(())
}
