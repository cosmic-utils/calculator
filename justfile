name := 'cosmic-ext-calculator'
appid := 'dev.edfloreshz.Calculator'

rootdir := ''
prefix := '/usr'
flatpak-prefix := '/app'

base-dir := absolute_path(clean(rootdir / prefix))
flatpak-base-dir := absolute_path(clean(rootdir / flatpak-prefix))SRPMS/cosmic/cosmic-ext-calculator-0.1.0-1.el10.src.rpm

export INSTALL_DIR := base-dir / 'share'

bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name
flatpak-bin-dst := flatpak-base-dir / 'bin' / name

desktop := appid + '.desktop'
desktop-src := 'res' / 'app.desktop'
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop
flatpak-desktop-dst := clean(rootdir / flatpak-prefix) / 'share' / 'applications' / desktop

metainfo := appid + '.metainfo.xml'
metainfo-src := 'res' / 'metainfo.xml'
metainfo-dst := clean(rootdir / prefix) / 'share' / 'metainfo' / metainfo
flatpak-metainfo-dst := clean(rootdir / flatpak-prefix) / 'share' / 'metainfo' / metainfo

icons-src := 'res' / 'icons' / 'hicolor'
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor'
flatpak-icons-dst := clean(rootdir / flatpak-prefix) / 'share' / 'icons' / 'hicolor'

# Default recipe which runs `just build-release`
default: build-release

# Runs `cargo clean`
clean:
    cargo clean

# Removes vendored dependencies
clean-vendor:
    rm -rf .cargo vendor vendor.tar

# `cargo clean` and removes vendored dependencies
clean-dist: clean clean-vendor

# Compiles with debug profile
build-debug *args:
    cargo build {{args}}

# Compiles with release profile
build-release *args: (build-debug '--release' args)

# Compiles release profile with vendored dependencies
build-vendored *args: vendor-extract (build-release '--frozen --offline' args)

# Runs a clippy check
check *args:
    cargo clippy --all-features {{args}} -- -W clippy::pedantic

# Runs a clippy check with JSON message format
check-json: (check '--message-format=json')

# Run the application for testing purposes
run *args:
    env RUST_BACKTRACE=full cargo run --release {{args}}

# Installs files
install:
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0644 {{desktop-src}} {{desktop-dst}}
    install -Dm0644 {{metainfo-src}} {{metainfo-dst}}
    install -Dm0644 "{{icons-src}}/scalable/apps/{{appid}}.svg" "{{icons-dst}}/scalable/apps/{{appid}}.svg"; \

# Installs flatpak files
flatpak:
    install -Dm0755 {{bin-src}} {{flatpak-bin-dst}}
    install -Dm0644 {{desktop-src}} {{flatpak-desktop-dst}}
    install -Dm0644 {{metainfo-src}} {{flatpak-metainfo-dst}}
    install -Dm0644 "{{icons-src}}/scalable/apps/{{appid}}.svg" "{{flatpak-icons-dst}}/scalable/apps/{{appid}}.svg"; \

# Uninstalls installed files
uninstall:
    rm {{bin-dst}} {{desktop-dst}} {{icons-dst}}

# Vendor dependencies locally
vendor:
    #!/usr/bin/env bash
    mkdir -p .cargo
    cargo vendor --sync Cargo.toml | head -n -1 > .cargo/config.toml
    echo 'directory = "vendor"' >> .cargo/config.toml
    echo >> .cargo/config.toml
    echo '[env]' >> .cargo/config.toml
    if [ -n "${SOURCE_DATE_EPOCH}" ]
    then
        source_date="$(date -d "@${SOURCE_DATE_EPOCH}" "+%Y-%m-%d")"
        echo "VERGEN_GIT_COMMIT_DATE = \"${source_date}\"" >> .cargo/config.toml
    fi
    if [ -n "${SOURCE_GIT_HASH}" ]
    then
        echo "VERGEN_GIT_SHA = \"${SOURCE_GIT_HASH}\"" >> .cargo/config.toml
    fi
    tar pcf vendor.tar .cargo vendor
    rm -rf .cargo vendor

# Extracts vendored dependencies
vendor-extract:
    rm -rf vendor
    tar pxf vendor.tar
