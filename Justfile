set shell := ["bash", "-o", "pipefail", "-c"]

cargo_features := ""
ci := ""

default:
    @just --list

# Run clippy with workspace lints and ESLint + svelte-check for frontend
lint:
    cargo clippy --workspace --all-targets
    cd frontend && bun run lint
    cd frontend && bunx svelte-check --tsconfig ./tsconfig.app.json --fail-on-warnings

# Run rustfmt and Prettier for frontend
fmt:
    cargo fmt --all
    cd frontend && bun run format

# Verify test compilation without running tests
check-compile:
    cargo test --workspace --no-run

# Run tests (memory-limited to 4GB to protect IDE from OOM, quiet unless failures)
test:
    @systemd-run --user --scope --quiet -p MemoryMax=4G -p MemorySwapMax=0 cargo test --workspace --quiet 2>&1 | awk '/^running [0-9]|^test result: ok|^[.]*$/{next} /FAILED/{failed=1} {print} END{exit failed+0}'

# Run tarpaulin for code coverage
coverage:
    cargo tarpaulin --workspace --skip-clean

# Run indexing benchmarks (criterion)
bench *ARGS:
    cargo bench --package flibrarian-core --features faking --bench indexing -- {{ ARGS }}

# Run iai-callgrind benchmarks (instruction-count regression guard)
bench-iai *ARGS:
    cargo bench --package flibrarian-core --features faking --bench indexing_iai -- {{ ARGS }}

# Build in release mode
build:
    cargo build --workspace --release

# Rust checks: fmt, lint, test compile, test
# Use `just cargo_features="--features bundled-duckdb" ci=1 check-rust` in CI
check-rust:
    @parallel --will-cite --compress --halt now,fail=1 --jobs 2 ::: \
        "cargo fmt --all -- --check" \
        "cargo clippy --workspace --all-targets {{ cargo_features }} --quiet" \
        "linecop crates"
    @cargo test --workspace {{ cargo_features }} --no-run --quiet
    @if [ -n "{{ ci }}" ]; then \
        cargo test --workspace {{ cargo_features }} --quiet 2>&1 | awk '/^running [0-9]|^test result: ok|^[.]*$/{next} /FAILED/{failed=1} {print} END{exit failed+0}'; \
    else \
        systemd-run --user --scope --quiet -p MemoryMax=4G -p MemorySwapMax=0 cargo test --workspace {{ cargo_features }} --quiet 2>&1 | awk '/^running [0-9]|^test result: ok|^[.]*$/{next} /FAILED/{failed=1} {print} END{exit failed+0}'; \
    fi

# Frontend checks: fmt, lint, svelte-check
check-frontend:
    @parallel --will-cite --compress --halt now,fail=1 --jobs 3 ::: \
        "cd frontend && bunx --silent prettier --check ." \
        "cd frontend && bun --silent run lint" \
        "cd frontend && bunx svelte-check --tsconfig ./tsconfig.app.json --fail-on-warnings" \
        "linecop frontend"

# Run all checks in parallel — only shows output on failure
check:
    @parallel --will-cite --compress --halt now,fail=1 --jobs 2 ::: \
        "just cargo_features='{{ cargo_features }}' ci='{{ ci }}' check-rust" \
        "just check-frontend"

# Regenerate all app icons (PNG, ICO, ICNS) from assets/icon.svg
icons:
    cargo tauri icon assets/icon.svg -o crates/flibrarian-gui/icons
    cp assets/icon.svg frontend/public/icon.svg
    rm -rf crates/flibrarian-gui/icons/android crates/flibrarian-gui/icons/ios
    rm -f crates/flibrarian-gui/icons/Square*.png crates/flibrarian-gui/icons/StoreLogo.png crates/flibrarian-gui/icons/64x64.png
    just icons-install

# Install icon + .desktop file for Wayland taskbar (dev use)
icons-install:
    mkdir -p ~/.local/share/icons/hicolor/128x128/apps
    cp crates/flibrarian-gui/icons/128x128.png ~/.local/share/icons/hicolor/128x128/apps/com.flibrarian.app.png
    mkdir -p ~/.local/share/icons/hicolor/scalable/apps
    cp assets/icon.svg ~/.local/share/icons/hicolor/scalable/apps/com.flibrarian.app.svg
    cp assets/com.flibrarian.app.desktop ~/.local/share/applications/
    gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor/ 2>/dev/null || true
    update-desktop-database ~/.local/share/applications/ 2>/dev/null || true

# Run Tauri GUI in development mode (hot-reloading frontend)
gui-dev:
    cargo tauri dev

# Build Tauri GUI for production with bundled DuckDB (deb + rpm; AppImage skipped — breaks on NixOS)
gui-build *ARGS:
    cargo tauri build --bundles deb,rpm {{ ARGS }}

# Run web server with frontend served from disk (for development)
web-dev:
    cd frontend && bun run build
    cargo run --bin flibrarian-web -- --frontend-dir frontend/dist
