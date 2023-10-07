set -ex
if [ ! -d fonts ]; then
    git clone  https://github.com/s3bk/pdf_fonts fonts
fi
cargo build
STANDARD_FONTS=$(pwd)/fonts target/debug/win-test
