#! /bin/bash -e

if [ -t 1 ]; then
    echo "Don't run this locally!  Run in docker using \`make test\` instead"
    exit 1
fi

git clone https://github.com/sebasmagri/env_logger
cd bitflags
srclib-rust scan --repo github.com/sebasmagri/env_logger --subdir .
echo "Test passed."