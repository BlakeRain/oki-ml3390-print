# Printing on Oki ML-3390

This repo contains two small programs for printing code to my Oki ML-3390: `escp-print` and `oki-ml3390-print`.

The first program, `escp-print`, will take some text input and format it for printing. This involves things like adding
a file header, adding line numbers, and syntax highlighting.

```
USAGE:
    escp-print [OPTIONS] [PATHS]...

ARGS:
    <PATHS>...

OPTIONS:
    -e, --extension <EXTENSION>
    -h, --header
        --help                     Print help information
    -t, --title <TITLE>
```

Note that if no file path(s) are passed to the program it will read from the standard input. In this case, the `-e` (or
`--extension`) option is useful to select the language for syntax highlighting.

The second program, `oki-ml3390-print` will print to an Oki ML-3390 found over a USB connection. This uses the `rusb`
crate to access the USB devices connected to the system to find the Oki ML-3390. It supports two printing modes: text
and binary.

```
USAGE:
    oki-ml3390-print [OPTIONS]

OPTIONS:
    -b, --binary       Read binary from stdin (useful for Epson escape data from GhostScript)
    -f, --form-feed    Add a form-feed to the end of the output (unavailable with `--binary`)
    -h, --help         Print help information
```

Typically to print some source code on the Oki I use the following command-line (which I have on an alias to brevity):

```
escp-print -h src/session_map.rs | oki-ml3390-print
```
