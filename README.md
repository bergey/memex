# memex

Enhanced search for Zotero and Calibre.

- not ()
- or
- only (has no tags besides those specified; need not have all of them)

See the test suite for full syntax & truth tables.  Here are some examples

``` sh
memex '(not rust)'
memex '(only rust)' # only rust or no tags
memex '(and rust (only rust))' # exactly one tag
memex '(only rust concurrency crabs)'
memex '(and rust (not safety))'
memex '"memory safety"' # quote tags with multiple words
memex '(and rust (not (or performance "memory safety")))'
```

## CLI arguments

```
$ memex --help
enhanced search for Zotero & Calibre

Usage: memex [OPTIONS] [QUERY]

Arguments:
  [QUERY]  

Options:
      --matches              print matching documents (default)
      --stats                print stats about library size
      --top <TOP>            print stats on this many common tags [default: 5]
  -v, --verbose              print what we're doing
      --library <LIBRARIES>  path to library; suppresses default search locations
  -o <OUTPUT_FILE>           output to file
  -h, --help                 Print help
  -V, --version              Print version
```
