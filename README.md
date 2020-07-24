# rockerize

D**ockerize** your **r**ust application easily.

It will build (and run) a minimal docker image for your application.

Built with rust.

## USAGE

```bash
# in your rust project's root,
$ cd /some/where/my-project/

# dockerize and run it,
$ rockerize

# or dockerize it only,
$ rockerize --build-only

# or dockerize it with exposed ports,
$ rockerize --exposed-ports 80 443

# or dockerize it with local files,
$ rockerize --add-files ./config.toml

# or just show the not-so-helpful help message
$ rockerize --help
```

## INSTALL

```bash
$ cargo install rockerize
```

## KNOWN ISSUES

- Not yet usable on platforms other than x64 (eg. armv7, arm64v8, ...) due to the [MUSL support](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html) [issue](https://github.com/rust-lang/docker-rust/issues/50).

## LICENSE

MIT

