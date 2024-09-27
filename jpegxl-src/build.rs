fn main() {
    let source = std::path::Path::new("libjxl");
    assert!(source.exists());
    assert!(source.is_dir());
}
