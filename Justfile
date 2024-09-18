set export

HTTP_PORT := "8090"
TRUNK_BRANCH := "master"
GIT_REMOTE := "origin"
NODEJS_PACKAGE_MANAGER := "npm"

# Help
list: 
    just --list

# Install rust dependencies
deps:
    set -e; \
    source ./funcs.sh; \
    check_deps ${NODEJS_PACKAGE_MANAGER}; \
    rustup target add wasm32-unknown-unknown; \
    cargo install --locked cargo-watch; \
    cargo install --locked trunk; \
    cargo install --locked git-cliff; \
    cargo install --locked cargo-edit; \
    cargo install --locked cargo-audit;

# Install rust depdencies (precompiled)
bin-deps:
    rustup target add wasm32-unknown-unknown
    cargo binstall -y --locked trunk
    cargo binstall -y --locked git-cliff

# Run (development)
run:
    cargo watch -s "sleep 8 && (killall dry_console || true) && just build && cargo run --bin dry_console -- -l debug --port ${HTTP_PORT} --open"

# Build frontend WASM (debug)
build-frontend: clean-dist
    cd frontend; npm ci; trunk build ${RELEASE_BUILD_ARGS:-} --filehash false

# Build frontend WASM (release)
build-release-frontend:
    RUSTFLAGS="-D warnings" RELEASE_BUILD_ARGS="--release" just build-frontend

# Build (debug)
build: build-frontend
    source ./funcs.sh; \
    check_emacs_unsaved_files;
    cargo build ${RELEASE_BUILD_ARGS:-}

# Build (release)
build-release:
    cd frontend; npm ci;
    RUSTFLAGS="-D warnings" RELEASE_BUILD_ARGS="--release" just build

# Install binary (to ~/.cargo/bin/)
install: deps build-release-frontend
    RUSTFLAGS="-D warnings" RELEASE_BUILD_ARGS="--release" cargo install --path server

# Run compiled release binary (no live-reload)
static-run: build-release
    ./target/release/dry_console --open

# bump release version
bump-version:
    @if [ -n "$(git status --porcelain)" ]; then echo "## Git status is not clean. Commit your changes before bumping version."; exit 1; fi
    @if [ "$(git symbolic-ref --short HEAD)" != "${TRUNK_BRANCH}" ]; then echo "## You may only bump the version from the ${TRUNK_BRANCH} branch."; exit 1; fi
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
    confirm yes "New version would be \"v${VERSION}\"" " -- Proceed?"; \
    git checkout -B release-v${VERSION}; \
    cargo set-version ${VERSION}; \
    sed -i "s/^VERSION=v.*$/VERSION=v${VERSION}/" README.md; \
    cargo update; \
    git add Cargo.toml Cargo.lock README.md; \
    git commit -m "release: v${VERSION}"; \
    echo "Bumped version: v${VERSION}"; \
    echo "Created new branch: release-v${VERSION}"; \
    echo "You should push this branch and create a PR for it."

release:
    @if [ -n "$(git status --porcelain)" ]; then echo "## Git status is not clean. Commit your changes before bumping version."; exit 1; fi
    @if [ "$(git symbolic-ref --short HEAD)" != "${TRUNK_BRANCH}" ]; then echo "## You may only release the ${TRUNK_BRANCH} branch."; exit 1; fi
    git remote update;
    @if [[ "$(git status -uno)" != *"Your branch is up to date"* ]]; then echo "## Git branch is not in sync with git remote ${GIT_REMOTE}."; exit 1; fi;
    @set -eo pipefail; \
    source ./funcs.sh; \
    CURRENT_VERSION=$(grep -Po '^version = \K.*' Cargo.toml | sed -e 's/"//g' | head -1); \
    if git rev-parse "v${CURRENT_VERSION}" >/dev/null 2>&1; then echo "Tag already exists: v${CURRENT_VERSION}"; exit 1; fi; \
    if (git ls-remote --tags "${GIT_REMOTE}" | grep -q "refs/tags/v${CURRENT_VERSION}" >/dev/null 2>&1); then echo "Tag already exists on remote ${GIT_REMOTE}: v${CURRENT_VERSION}"; exit 1; fi; \
    cargo audit | less; \
    confirm yes "New tag will be \"v${CURRENT_VERSION}\"" " -- Proceed?"; \
    git tag "v${CURRENT_VERSION}"; \
    git push "${GIT_REMOTE}" tag "v${CURRENT_VERSION}";

# cleans ./dist
clean-dist:
    rm -rf dist

# cleans ./release
clean-release:
    rm -rf release

# cleans ./frontend/node_modules
clean-node-modules:
    rm -rf frontend/node_modules

# clean all artifacts
clean: clean-dist clean-release clean-node-modules
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

# Run clippy linter and paginate results with less
clippy:
    RUSTFLAGS="-D warnings" cargo clippy --color=always 2>&1 | less -R

# Run clippy linter and apply fixes
clippy-fix:
    RUSTFLAGS="-D warnings" cargo clippy --fix --color=always 2>&1 | less -R

# Run the binary with the --help argument:
help: build
    cargo run --bin dry_console -- --help
