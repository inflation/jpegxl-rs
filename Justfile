default: test doc commit

test:
    cargo nextest run -F vendored
    cargo test --doc -F vendored

doc:
    cargo readme -r jpegxl-rs > README.md
    cd jpegxl-rs && gitmoji-changelog --preset cargo
    mv jpegxl-rs/CHANGELOG.md .

commit:
    git commit -a