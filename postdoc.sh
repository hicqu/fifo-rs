#!/usr/bin/env bash
echo '<meta http-equiv=refresh content=0;url=YOURLIBNAME/index.html>' > target/doc/index.html
ghp-import -p target/doc
