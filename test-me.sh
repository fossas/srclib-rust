#! /bin/bash -e

if [ -t 1 ]; then
    echo "Don't run this locally!  Run in docker using \`make test\` instead"
    exit 1
fi

git clone https://github.com/scruffystuffs/msh.git
cd msh
srclib-rust scan --repo github.com/scruffystuffs/msh --subdir .
cargo metadata --offline --no-deps
echo "Test passed."