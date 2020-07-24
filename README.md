# rockerize

D**ockerize** your **r**ust application easily.

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

## LICENSE

MIT

