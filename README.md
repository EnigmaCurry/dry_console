# dry_console

This is a full stack Rust web app using [axum](https://github.com/tokio-rs/axum) and [yew](https://yew.rs/). 

It's purpose has not yet been fully defined, but it's gonna be a
docker helper tool for [d.rymcg.tech](d.rymcg.tech)

# Platforms

This tool is built for the following platforms:

 * Linux x86_64 (AMD64)
 * Linux aarch64 (ARM64)
 * MacOS x86_64 (AMD64)
 * MacOS aarch64 (ARM64)

[Download the latest release for your platform.](https://github.com/EnigmaCurry/dry_console/releases)

For MS Windows, use the Linux version inside WSL2.

# Install script

To install, copy and paste this entire code block directly into your
Bash shell. (Customize the variables at the top, if you wish):

```
# Cross platform Bash install script for dry_console:

# Configure the version to download:
VERSION=v0.1.38
PLATFORM=$(uname -s)-$(uname -m)
TMP_DIR=$(mktemp -d)

# Download and extract the release tarball:
(set -e
mkdir -p ${TMP_DIR}
cd ${TMP_DIR}
curl -LO https://github.com/EnigmaCurry/dry_console/releases/download/${VERSION}/dry_console-${VERSION}-${PLATFORM}.tar.gz
tar xfv dry_console-${VERSION}-${PLATFORM}.tar.gz
rm -f dry_console-${VERSION}-${PLATFORM}.tar.gz)

# Change directory to TMP_DIR and show the extracted program:
cd ${TMP_DIR}
pwd
ls -lh
```

This script will create a temporary directory (`TMP_DIR`) and download
and extract the release tarball specific to your platform. The program
is a single, self-contained binary. To start the program, simply run
`dry_console` from the temporary directory:

```
./dry_console
```

To install the program system wide, run:

```
sudo install ./dry_console /usr/local/bin
```

With the program installed in `/usr/local/bin` (which should already
be included in your `PATH`), you may now run the program from any
working directory (ie. without specifying the `./` in front):

```
dry_console
```

# Development
## Dependencies

 * Rust and Cargo installed via [rustup](https://rustup.rs/).

 * [Just](https://github.com/casey/just?tab=readme-ov-file#readme)
 
```
cargo install --locked just
```

Install the rest of the dependencies, using the `just` target:

```
just deps
```

Install `dry_console`:

```
just install
```

(`dry_console` is now installed in `~/.cargo/bin`, which should be
added to your shell's `PATH` variable.)

## Run development server

```
just run
```

## Release (Github actions)

### Bump release version

Update the version number in Cargo.toml, Cargo.lock, and README.md as
suggested by git-cliff:

```
just bump-version
```

### Make PR with the changeset

Branch protection is enabled, all changesets must come in the form of
a Pull Request.

### Merge PR and tag release

Once the PR is merged, tag the release `vX.X.X` and push it to the
`master` branch.

```
git checkout master
git pull origin master
git tag vX.X.X
git push origin tag vX.X.X
```

