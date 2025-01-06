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
