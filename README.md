# srclib-rust
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Ffossas%2Fsrclib-rust.svg?type=shield)](https://app.fossa.io/projects/git%2Bgithub.com%2Ffossas%2Fsrclib-rust?ref=badge_shield)

srclib (srclib.org) toolchain for Rust with Cargo package manage support.


## Known Issues

- Building on Mac OS X requires OpenSSL work around:

```
export CFLAGS='-I/usr/local/opt/openssl/include -L/usr/local/opt/openssl/lib'
```

## License
[![FOSSA Status](https://app.fossa.io/api/projects/git%2Bgithub.com%2Ffossas%2Fsrclib-rust.svg?type=large)](https://app.fossa.io/projects/git%2Bgithub.com%2Ffossas%2Fsrclib-rust?ref=badge_large)