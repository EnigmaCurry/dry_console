# Help
list: 
    just --list

# Install rust dependencies
deps:
    rustup target add wasm32-unknown-unknown
    cargo install --locked cargo-watch
    cargo install --locked trunk
    cargo install --locked git-cliff
    cargo install --locked cargo-edit
    
# Install rust depdencies (precompiled)
bin-deps:
    rustup target add wasm32-unknown-unknown
    cargo binstall -y --locked trunk
    cargo binstall -y --locked git-cliff

# Run (development)
run:
    cargo watch -s 'just build && cargo run --bin dry_console -- --port 8080'

# Build frontend WASM (debug)
build-frontend: clean-dist
    cd frontend; trunk build ${RELEASE_BUILD_ARGS:-} --filehash false

# Build frontend WASM (release)
build-release-frontend:
    RUSTFLAGS="-D warnings" RELEASE_BUILD_ARGS="--release" just build-frontend

# Build (debug)
build: build-frontend
    cargo build ${RELEASE_BUILD_ARGS:-}

# Build (release)
build-release:
    RUSTFLAGS="-D warnings" RELEASE_BUILD_ARGS="--release" just build

# Install binary (to ~/.cargo/bin/)
install: deps build-release-frontend
    RUSTFLAGS="-D warnings" RELEASE_BUILD_ARGS="--release" cargo install --path server

# Run compiled release binary (no live-reload)
static-run: build-release
    ./target/release/server

# bump release version
bump-version:
    @if [ -n "$(git status --porcelain)" ]; then echo "## Git status is not clean. Commit your changes before bumping version."; exit 1; fi
    source ./funcs.sh; \
    set -eo pipefail; \
    CURRENT_VERSION=$(grep -Po '^version = \K.*' Cargo.toml | sed -e 's/"//g' | head -1); \
    VERSION=$(git cliff --bumped-version | sed 's/^v//'); \
    echo; \
    (if git rev-parse v${VERSION} 2>/dev/null; then \
      echo "New version tag already exists: v${VERSION}" && \
      echo "If you need to re-do this release, delete the existing tag (git tag -d v${VERSION})" && \
      exit 1; \
     fi \
    ); \
    echo "## Current $(grep '^version =' Cargo.toml | head -1)"; \
    confirm yes "New version would be \"${VERSION}\"" " -- Proceed?"; \
    cargo set-version ${VERSION}; \
    sed -i "s/^VERSION=v.*$/VERSION=v${VERSION}/" README.md; \
    cargo update; \
    git add Cargo.toml Cargo.lock README.md; \
    git commit -m "release: ${VERSION}"; \
    echo "Bumped version: ${VERSION}"; \

# # self-hosted release (non-github actions)
# release: clean-dist build-release
#     rm -rf release; \
#     TMP_DIR=$(mktemp -d); \
#     VERSION=$(cd server; cargo read-manifest | jq -r .version); \
#     PACKAGE=dry_console-v${VERSION}; \
#     PACKAGE_DIR=${TMP_DIR}/${PACKAGE}; \
#     PACKAGE_PATH=${TMP_DIR}/${PACKAGE}.tar.gz; \
#     mkdir ${PACKAGE_DIR}; \
#     cp -r ./target/release/dry_console ${PACKAGE_DIR}; \
#     (cd ${TMP_DIR}; tar cfz ${PACKAGE}.tar.gz ${PACKAGE}); \
#     mkdir -p release; \
#     cp ${PACKAGE_PATH} release/; \
#     (cd release; tar xfvz ${PACKAGE}.tar.gz); \
#     rm -rf ${TMP_DIR};

# cleans ./dist
clean-dist:
    rm -rf dist

# cleans ./release
clean-release:
    rm -rf release

# clean all artifacts
clean: clean-dist clean-release
    cargo clean

# Install dry_console as a systemd service
systemd-install: install
    mkdir -p ${HOME}/.config/systemd/user
    cp systemd.service ${HOME}/.config/systemd/user/dry_console.service

# Enable dry_console systemd service
systemd-enable: systemd-install
    systemctl --user enable --now dry_console
    systemctl --user status dry_console --no-pager

# Disable dry_console systemd service
systemd-disable: 
    systemctl --user disable --now dry_console --no-pager
    systemctl --user status dry_console --no-pager

# Restart dry_console systemd service
systemd-restart: 
    systemctl --user restart --force dry_console
    systemctl --user status dry_console --no-pager
