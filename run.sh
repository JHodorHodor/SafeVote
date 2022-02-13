# Usage: ./run.sh <log_level> <n_voters> <threshold> <options>
# after - taskkill cargo process

cargo build

cargo run -p voting-server -- $2 $3 $4 &

for (( i=0; i<$2; i++ ))
   do
       RUST_LOG=$1 cargo run -p voting-system -- $i 2>&1 | tee out$i.txt & 
   done
