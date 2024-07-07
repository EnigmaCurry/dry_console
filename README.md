# dry_console

This is a full stack Rust web app using [axum](https://github.com/tokio-rs/axum) and [yew](https://yew.rs/). It was created from a [cargo-generate](https://cargo-generate.github.io/cargo-generate/) template: `cargo generate rksm/axum-yew-template`.

It's purpose has not been fully defined, but it's gonna be a docker helper tool for [d.rymcg.tech](d.rymcg.tech)

# Install

[Download the latest release for your platform.](https://github.com/EnigmaCurry/dry_console/releases)

This tool is built for the following platforms:

 * Linux x86_64 (AMD64)
 * Linux aarch64 (ARM64)
 * MacOS x86_64 (AMD64)
 * MacOS aarch64 (ARM64)

For MS Windows, use the Linux version in WSL2.

```
VERSION=v0.1.38
PLATFORM=$(uname -s)-$(uname -m)
INSTALL_PATH=~/dry_console

(set -e
mkdir -p ${INSTALL_PATH}
cd ${INSTALL_PATH}
curl -LO https://github.com/EnigmaCurry/dry_console/releases/download/${VERSION}/dry_console-${VERSION}-${PLATFORM}.tar.gz
tar xfv dry_console-${VERSION}-${PLATFORM}.tar.gz
rm -f dry_console-${VERSION}-${PLATFORM}.tar.gz)

cd ${INSTALL_PATH}
pwd
ls
```

The script printed above will create a new directory (`INSTALL_PATH`)
and it will download and extract the dry_console version specified
(`VERSION`). To start the program, simply run `dry_console` from that
directory.

```
./dry_console
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

## Production

Build and run the production binary:

```
just static-run
```

Clean build and release a new package:

```
## release is a minimal release target for self-hosting:
## (This is not what github actions uses)
just release
```

A wild tarball appears in `./release`.

## Bump release version

Do this before releasing a new version, it will update Cargo.toml,
Cargo.lock, and README.md with the new version suggested by git-cliff:

```
just bump-version
```
