default: test doc 

test:
    cargo nextest run -F vendored
    cargo test --doc -F vendored

doc:
    cargo readme -r jpegxl-rs > README.md

release: test doc 
    cd jpegxl-rs && gitmoji-changelog --preset cargo --output ../CHANGELOG.md
    git commit -am "🔖 v$(cargo metadata --format-version=1 | \
        jq -r '.packages[] | select(.name | contains("jpegxl-rs")) | .version')"

publish:
    cargo publish -p jpegxl-src
    cargo publish -p jpegxl-sys -F vendored
    cargo publish -p jpegxl-rs -F vendored
