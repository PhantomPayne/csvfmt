# csvfmt

A fast, composable command-line tool that applies a **format string** to every row of CSV, TSV, or any other delimited data.

## Installation

```sh
cargo install --path .
```

Or build directly:

```sh
cargo build --release
# binary is at ./target/release/csvfmt
```

## Quick start

```sh
# Basic substitution – fields are 1-based
echo "Alice,30" | csvfmt "Hello {1}, you are {2} years old."
# → Hello Alice, you are 30 years old.

# Works great with pbpaste / clipboard data
pbpaste | csvfmt "Hello {1}, how are you {2}?"
```

## Format string syntax

| Syntax | Meaning |
|---|---|
| `{N}` | Value of the Nth field (1-based) |
| `{name}` | Value of the field whose header is `name` (requires `-H`) |
| `{N:default}` | Field value, or `default` when the field is empty |
| `{?N:text}` | Include `text` only when field N is **non-empty**; `text` may itself contain `{…}` references |
| `{{` / `}}` | Literal `{` / `}` |

## Options

```
USAGE:
    csvfmt [OPTIONS] <TEMPLATE>

ARGS:
    <TEMPLATE>    Format template to apply to each row

OPTIONS:
    -i, --input <FILE>        Input file (default: stdin)
    -d, --delimiter <CHAR>    Field delimiter (default: `,`); use `\t` or `tab` for tab
    -t, --tsv                 Shorthand for `-d '\t'`
    -H, --header              Treat the first row as a header row (enables {name} references)
        --trim                Trim leading/trailing whitespace from each field
    -s, --skip-empty          Skip rows where every field is empty
    -h, --help                Print help
    -V, --version             Print version
```

## Examples

### Named fields with a header row

```sh
printf "first,last,email\nAlice,Smith,alice@example.com\n" \
  | csvfmt -H "Dear {first} {last} <{email}>,"
# → Dear Alice Smith <alice@example.com>,
```

### Conditional blocks

Include text only when a field has a value:

```sh
echo "Alice,30,Engineer" | csvfmt "Hello {1}{?3:, job: {3}}"
# → Hello Alice, job: Engineer

echo "Alice,30,"       | csvfmt "Hello {1}{?3:, job: {3}}"
# → Hello Alice
```

### Default values

```sh
echo "Alice,," | csvfmt "{1} is {2:unknown} years old"
# → Alice is unknown years old
```

### TSV input

```sh
cat data.tsv | csvfmt --tsv "{1}: {2}"

# Or read from a file directly:
csvfmt --tsv -i data.tsv "{1}: {2}"
```

### Custom delimiter

```sh
echo "Alice;30;Berlin" | csvfmt -d ';' "{1} lives in {3}"
# → Alice lives in Berlin
```

### Skip blank rows & trim whitespace

```sh
printf "Alice,30\n\n,\nBob,25\n" | csvfmt --skip-empty "{1}: {2}"
# → Alice: 30
# → Bob: 25

echo " Alice , 30 " | csvfmt --trim "Name={1}, Age={2}"
# → Name=Alice, Age=30
```

### Generate SQL INSERT statements

```sh
cat users.csv | csvfmt "INSERT INTO users (name, age) VALUES ('{1}', {2});"
```

### Generate markdown links

```sh
# CSV: title,url
cat links.csv | csvfmt "[{1}]({2})"
```

### Escaped braces

```sh
echo "Alice" | csvfmt "{{{1}}}"
# → {Alice}
```

## License

MIT
