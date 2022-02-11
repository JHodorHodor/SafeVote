cargo build
cargo run -p voting-server -- A,B &
RUST_LOG=$1 cargo run -p voting-system -- 0 & 
RUST_LOG=$1 cargo run -p voting-system -- 1 &
RUST_LOG=$1 cargo run -p voting-system -- 2 &
