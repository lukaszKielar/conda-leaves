# conda-leaves

Simple CLI tool that allows to pretty print all dependencies within conda environment.

## Installation

`conda-leaves` doesn't require any additional libraries to be installed, but it assumes user has `conda` and `cargo` installed and available in system's path.

Installation is as simple as running `cargo install conda-leaves`.

## Commands

### `help`

Prints help information.

```bash
conda-leaves help
```

### `leaves`

Prints top level packages in conda environment.

Flags:

- `--no-pip` - Prints packages installed by conda only

Usage:

```bash
conda-leaves leaves [Flags]
```

### `package`

Prints tree view for the package.

Options:

- `-n`, `-name` - Name of the package that should be printed.

Usage:

```bash
conda-leaves package --name <name>
```

### `export`

Exports leaves to the file.

Options:

- `-f`, `--filename` (default: environment.yml) - Name of the output yml file.

Usage:

```bash
conda-leaves export [Options]
```

## Development

### Running CLI using test data

```bash
CONDA_PREFIX="./tests/data" cargo run --release -- <command>
```

### Running tests

```bash
cargo test
```

### Compiling documentation

```bash
cargo doc
```
