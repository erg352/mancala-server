# Mancala Server

Server used to match-make different mancala bots via HTTP.

## Requirements

- [GNU Make](https://www.gnu.org/software/make/): This is optional, and is used
to easily build and run the program. This is not strictly speaking required, and
methods of building and running are described in the "Running" section.
- [sqlite](https://www.sqlite.org/index.html): This is required, as all persistant
data will be stored in a sqlite database.
- [rust](https://www.rust-lang.org/): The rust compiler, it's build tool cargo
and several other bundled binaries are required to run the server. This project is built
in rust, and this repo only contains the source code, so it can not be used without this
dependency.
- [trunk](https://trunkrs.dev/): This is optional, and must be present for the frontend
to be built. It also requires rust and cargo to be installed on your machine.

## Installation

Run `git clone https://github.com/erg352/mancala-server.git --recursive-submodules`. If the
`recursive-submodules` path was not specified during the `git clone`, simply run `git submodule init`
and `git submodule update`.

## Running

**Make sure the repo is properly installed, including it's submodules**. Run
`make build-release` to build the project, an `make run-release` to build and run it.

If [GNU Make](https://www.gnu.org/software/make/) is not installed on the machine, the
following commands can also be ran:

### Building:
```bash
cargo build --release
# Required if we want to build the frontend as well, but
# can be omitted if this is not needed. This requires trunk,
# see the dependencies section for more information.
(cd frontend && trunk build --release)
```

### Running:
```bash
# If we are intent on using the frontend generated by trunk
cargo run --release -- --port $(MY_PORT) --database $(MY_DATABASE_PATH) --static-routes frontend/dist

# If we do not care about the frontend.
cargo run --release -- --port $(MY_PORT) --database $(MY_DATABASE_PATH)
```
Where `MY_PORT` and `MY_DATABASE_PATH` corresponding to the desired port and sqlite database file the server should
bind to. Run `cargo run release -- --help` for more information about the specific arguments the program can take.

Instead of running `cargo run --release --`, which will automaticaly rebuild the project (if any changes are found)
each time, we can instead simply call the executable found in `target/release/match-server`. This can be
symlinked to anywhere that's more practical to access. If we run the program this way, the `--` found after `--release`
must be omitted, as it's simply used by cargo to split cargo's commands with the called program's commands.
