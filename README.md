# dry_console

A workstation tool for managing remote Docker instances and
applications, deployed by [d.rymcg.tech](d.rymcg.tech).

This application is implemented as a local web service, which opens
automatically in your preferred web browser. Security of the
application relies upon a one-time password, which is randomized on
each startup, and then it disables the login service to prevent future
attempts. Therefore, the cookie assigned to your web browser becomes
the only authorized client key. To login again, you need to restart
the service.

# Platforms

This tool is built for the following platforms:

 * Linux x86_64 (AMD64)
 * Linux aarch64 (ARM64)
 * MacOS x86_64 (AMD64)
 * MacOS aarch64 (ARM64)

[Download the latest release for your platform.](https://github.com/EnigmaCurry/dry_console/releases)

For MS Windows, use the Linux version inside WSL2.

# Install script

The release is self-contained in a single binary, so you may install it however you like.
To automate the install system wide, you may copy and paste this *entire* code block 
directly into your Bash shell. (Customize the variables at the top, beforehand, 
if you wish):

```
# Cross platform Bash install script for dry_console:
(set -ex

# Configure these variables if you wish:
VERSION=v0.2.0
INSTALL_DIR=/usr/local/bin
REPO_DOWNLOAD=https://github.com/EnigmaCurry/dry_console/releases/download
USE_SUDO=true

# Download and extract the platform specific release tarball:
PLATFORM=$(uname -s)-$(uname -m)
PROGRAM=dry_console
TMP_DIR=$(mktemp -d)
if [[ "${USE_SUDO}" == "true" ]]; then
    SUDO_PREFIX="sudo"
else
    SUDO_PREFIX=""
fi
mkdir -p ${TMP_DIR}
pushd ${TMP_DIR}
curl -L ${REPO_DOWNLOAD}/${VERSION}/${PROGRAM}-${VERSION}-${PLATFORM}.tar.gz \
     -o release.tar.gz
tar xfv release.tar.gz
${SUDO_PREFIX} install ${TMP_DIR}/${PROGRAM} ${INSTALL_DIR}
popd
rm -rf ${TMP_DIR}
ls -lh ${INSTALL_DIR}/${PROGRAM})
```

By default, this script uses `sudo` to install the binary to your
chosen `INSTALL_DIR` (`/usr/local/bin` by default). It may prompt you
to enter your password as it does this. If you don't need to use
`sudo`, set `USE_SUDO=false`.

# Development

dry_console is a full stack Rust web app using [axum](https://github.com/tokio-rs/axum) and [yew](https://yew.rs/). 

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

## Credits

This project was initialized from a starter project:
[rksm/axum-yew-setup](https://github.com/rksm/axum-yew-setup), used by
permission, see [LICENSE](LICENSE).

