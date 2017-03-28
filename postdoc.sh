#!/usr/bin/env bash
cargo doc --release
echo '<meta http-equiv=refresh content=0;url=fifo/index.html>' > target/doc/index.html
ghp-import -p target/doc
