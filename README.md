# 🧹 tidy-json

A CLI tool for sorting JSON files.

## Install

```sh
brew install todor-a/tap/tidy-json 
```

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/todor-a/tidy-json/releases/latest/download/tidy-json-installer.sh | sh
```

## Usage

```sh
tidy-json **/*.json --write
```

Check mode (CI-friendly):

```sh
tidy-json **/*.json --check
```

Read from stdin:

```sh
cat package.json | tidy-json --stdin
```

Print file results to stdout without writing:

```sh
tidy-json **/*.json --stdout
```

Use defaults from `.tidy-json.toml`:

```toml
write = true
order = "asc"
indent = 2
indent_style = "spaces"
```

## Options
```
Usage: tidy-json [OPTIONS] <INCLUDE>...

Arguments:
  <INCLUDE>...  File patterns to process (e.g., *.json)

Options:
  -e, --exclude <EXCLUDE>            File patterns to exclude (e.g., *.json)
  -w, --write                        Write the sorted JSON back to the input files
      --check                        Check if files would change without writing them
  -b, --backup                       Create backups before modifying files
  -d, --depth <DEPTH>                Specify how deep the sorting should go
  -o, --order <ORDER>                Specify the sort order [default: asc] [possible values: asc, desc, rand, key-length-asc, key-length-desc, line-length]
  -i, --indent <INDENT>              Specify the desired indent
      --indent-style <INDENT_STYLE>  Specify the desired indent style [possible values: tabs, spaces]
      --stdin                        Read input from stdin instead of files
      --stdout                       Print sorted output to stdout
      --config <CONFIG>              Path to a TOML config file
      --log-level <LOG_LEVEL>        Specify log level [possible values: quiet, default, verbose]
  -h, --help                         Print help
  -V, --version                      Print version
```

## Parsing behavior

`tidy-json` parses standard JSON and also accepts trailing commas.

## Example

### `$ tidy-json **/*.json`
<table>
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```json
{
  "b": 1,
  "a": 2,
  "c": 3
}
```
  
</td>
<td>

```json
{
  "a": 2,
  "b": 1,
  "c": 3
}
```

</td>
</tr>
</table>

### `$ tidy-json **/*.json --depth=1`
<table>
<tr>
<th>Before</th>
<th>After</th>
</tr>
<tr>
<td>

```json
{
  "b": 1,
  "a": {
    "b": 1,
    "a": 2,
    "c": 3
  },
  "c": 3
}
```
  
</td>
<td>

```json
{
  "a": {
    "b": 1,
    "a": 2,
    "c": 3
  },
  "b": 1,
  "c": 3
}
```

</td>
</tr>
</table>
