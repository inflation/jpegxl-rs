default: test doc 

test:
    cargo nextest run --all-features
    cargo test --doc --all-features

doc:
    cargo readme -r jpegxl-rs > README.md

release: test doc 
    cd jpegxl-rs && gitmoji-changelog --preset cargo --output ../CHANGELOG.md
    git commit -am "🔖 v$(cargo metadata --format-version=1 | \
        jaq -r '.packages[] | select(.name | contains("jpegxl-rs")) | .version')"

publish:
    cargo publish -p jpegxl-src
    cargo publish -p jpegxl-sys --all-features
    cargo publish -p jpegxl-rs --all-features
