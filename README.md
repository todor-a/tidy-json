# 🧹 json-tidy

A cli tool for sorting JSON files.

## Install

```sh
brew install todor-a/tap/tidy-json 
```

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/todor-a/tidy-json/releases/download/v0.1.0/tidy-json-installer.sh | sh
```

## Usage

```sh
tidy-json **/*.json --write
```

## Options

```
Usage: tidy-json [OPTIONS] <INCLUDE>...

Arguments:
  <INCLUDE>...  File patterns to process (e.g., *.json)

Options:
  -e, --exclude <EXCLUDE>  File patterns to exclude (e.g., *.json)
  -w, --write              Write the sorted JSON back to the input files
  -b, --backup             Create backups before modifying files
  -f, --ignore-git-ignore  Whether the files specified in .gitignore should also be sorted
  -o, --order <ORDER>      Specify the sort order [default: asc] [possible values: asc, desc, rand]
  -h, --help               Print help
  -V, --version            Print version
```