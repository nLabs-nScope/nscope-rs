use nscope;

fn main() {
    let nscope_ctx = nscope::Context::new().expect("Failed to initialize nscope::Context");
    nscope::available_nscopes(&nscope_ctx);
}