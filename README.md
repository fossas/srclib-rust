# srclib-rust
srclib (srclib.org) toolchain for Rust with Cargo package manage support.


## Known Issues

- Building on Mac OS X requires OpenSSL work around:

```
export CFLAGS='-I/usr/local/opt/openssl/include -L/usr/local/opt/openssl/lib'
```