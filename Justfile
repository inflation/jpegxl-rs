default: test doc 

test:
    cargo nextest run -F vendored
    cargo test --doc -F vendored

doc:
    cargo readme -r jpegxl-rs > README.md

release: test doc 
    cd jpegxl-rs && gitmoji-changelog --preset cargo --output ../CHANGELOG.md