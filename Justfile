default: test doc commit

test:
    cargo nextest run -F vendored
    cargo test --doc -F vendored

doc:
    cargo readme -r jpegxl-rs > README.md

commit:
    git commit -a

release: test doc commit
    cd jpegxl-rs && gitmoji-changelog --preset cargo --output ../CHANGELOG.md