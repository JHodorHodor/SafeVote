cargo build
cargo run -p voting-server -- 1,2,3 &
cargo run -p voting-system -- 0 & 
cargo run -p voting-system -- 1 &
cargo run -p voting-system -- 2 &
