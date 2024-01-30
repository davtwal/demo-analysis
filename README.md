# demo-analysis

![Build Status](https://github.com/davtwal/demo-analysis/workflows/CI/badge.svg)

Analyzes and allows for basic viewing of TF2 .dem (Demo) files.

Utilizes [tf_demo_parser](https://github.com/demostf/parser) and [pyo3](https://github.com/PyO3/pyo3) to load the files in Rust and analyze the data contained within in Python.

## Building

This project uses rust and requires `cargo` and friends, see [the rust website](https://www.rust-lang.org/)
for how to get started.

Python 3.8+ and `maturin` are also required, and it is recommended to install [pyenv][1] (or [pyenv-win][1] for windows) and utilize a virtual environment.

### Recommended Setup

1. Clone the respository and `cd` into it
2. Install all items required for Rust
3. Install pyenv and set up Python 3.8 or higher:
    1. For Windows, install [pyenv-win][1], ideally using powershell
    2. For Linux or MacOS, install [pyenv][1]
4. Install [virtualenv][3] using `pip install virtualenv` and create a virtual environment:
    1. Create a virtual environment using `virtualenv .env`
    2. Enable the virtual environment:
        1. Windows: `.\.env\Scripts\activate`
        2. Linux or MacOS: `source .env/bin/activate`
    3. Note: The virtual environment can be deactivated by replacing `activate` with `deactivate`
5. Install [maturin][4] using `pip install maturin`
6. Build the python library using `maturin develop`
7. Run the executable using `cargo run`

## Executable Usage

```plaintext
demo_analysis.exe [OPTIONS] [FILENAMES]...

Arguments:
  [FILENAMES]...  The demo files to parse. If viewing with a window, only the first will be parsed.

Options:
  -a             Automatically analyze the given files.
  -n             Disable demo viewing. If -a is not specified, the executable does nothing.
  -h, --help     Print help
  -V, --version  Print version
```

## Performing Custom Analysis

When the executable wants to perform analysis, it calls pre-determined functions inside of `python/demo_analysis.py`. These are as follows:

- `demo_analysis_main` is called to perform analysis on the demo file as a whole
- `tick_analysis_main` is called to perform analysis on a single game tick
- `generate_grouping` generates groupings of players.

To see what types are exposed to the python script, see `python/demo_analysis_lib/*.pyi` to see the interfaces.

[1]: https://github.com/pyenv/pyenv
[2]: https://github.com/pyenv-win/pyenv-win
[3]: https://virtualenv.pypa.io/en/latest/index.html
[4]: https://pypi.org/project/maturin/
