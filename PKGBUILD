# Maintainer: Kay <s0990006@gmail.com>
pkgname=nautilus-image-converter-reborn-app
pkgver=0.1.0
pkgrel=1
pkgdesc="Standalone app to convert, resize, and rotate images — built with Rust and GTK4"
arch=('x86_64' 'aarch64')
url="https://github.com/Kayxue/nautilus-image-converter-reborn-app"
license=('MIT')
depends=(
    'gtk4'
    'libx11'   # gdk4-x11 / xlib
    'wayland'  # gdk4-wayland
)
makedepends=(
    'rust'
    'cargo'
)
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')  # Replace with actual checksum once a tagged release exists

prepare() {
    cd "$srcdir/$pkgname-$pkgver"

    # Pre-fetch Cargo dependencies into the offline cache so the build step
    # does not need network access (required for Arch packaging guidelines).
    export CARGO_HOME="$srcdir/cargo-home"
    cargo fetch --locked --target "$(rustc -vV | sed -n 's|host: ||p')"
}

build() {
    cd "$srcdir/$pkgname-$pkgver"

    export CARGO_HOME="$srcdir/cargo-home"
    export RUSTFLAGS="-C opt-level=2 -C link-arg=-Wl,-z,relro,-z,now"

    cargo build --frozen --release
}

check() {
    cd "$srcdir/$pkgname-$pkgver"

    export CARGO_HOME="$srcdir/cargo-home"
    cargo test --frozen --release
}

package() {
    cd "$srcdir/$pkgname-$pkgver"

    # Install the compiled binary
    install -Dm755 "target/release/nautilus_image_converter" \
        "$pkgdir/usr/bin/nautilus-image-converter"

    # Install the license
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

    # Install the README as documentation
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
