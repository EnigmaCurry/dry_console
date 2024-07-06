list:
    just --list

deps:
    rustup target add wasm32-unknown-unknown
    cargo install --locked cargo-watch
    cargo install --locked trunk

bin-deps:
    rustup target add wasm32-unknown-unknown
    cargo binstall -y --locked trunk
    cargo binstall -y --locked git-cliff

run:
    cargo watch -s 'just build && cargo run --bin dry_console -- --port 8080 --live-reload'

build-frontend: clean-dist
    cd frontend; trunk build ${RELEASE_BUILD_ARGS:-} --filehash false

build: build-frontend
    cargo build ${RELEASE_BUILD_ARGS:-}

build-release:
    RUSTFLAGS="-D warnings" RELEASE_BUILD_ARGS="--release" just build

install: build-release-frontend
    RUSTFLAGS="-D warnings" RELEASE_BUILD_ARGS="--release" cargo install --path server

build-release-frontend:
    RUSTFLAGS="-D warnings" RELEASE_BUILD_ARGS="--release" just build-frontend

static-run: build-release
    ./target/release/server

release: clean-dist build-release
    rm -rf release; \
    TMP_DIR=$(mktemp -d); \
    VERSION=$(cd server; cargo read-manifest | jq -r .version); \
    PACKAGE=dry_console-${VERSION}; \
    PACKAGE_DIR=${TMP_DIR}/${PACKAGE}; \
    PACKAGE_PATH=${TMP_DIR}/${PACKAGE}.tar.gz; \
    mkdir ${PACKAGE_DIR}; \
    cp -r ./target/release/server ./dist ${PACKAGE_DIR}; \
    (cd ${TMP_DIR}; tar cfz ${PACKAGE}.tar.gz ${PACKAGE}); \
    mkdir -p release; \
    cp ${PACKAGE_PATH} release/; \
    (cd release; tar xfvz ${PACKAGE}.tar.gz); \
    rm -rf ${TMP_DIR};

clean-dist:
    rm -rf dist

clean-release:
    rm -rf release
    
clean: clean-dist clean-release
    cargo clean
