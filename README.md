# cdio-utils-rs: Utility programs of the Compact Disc (CD) Input and Control Core Library

This is a WIP Rust port of the CLI utilities provided by libcdio.
Also includes [libcdio-rs](./libcdio-rs), which provides safe Rust abstractions
over libcdio.

## Status
 * [ ] cd-drive:      show CD-ROM drive characteristics
 * [ ] cd-info:       show information about a CD or CD-image
 * [ ] cd-paranoia:   an audio CD ripper
 * [ ] cd-read:       read information from a CD or CD-image
 * [ ] cdda-player:   a simple curses-based audio CD player
 * [ ] iso-info:      show information about an ISO 9660 image
 * [ ] iso-read:      read portions of an ISO 9660 image
 * [ ] mmc-tool:      issue low-level commands to a CD drive

## Development
### Build Requirements
- [Cargo](https://rust-lang.org/learn/get-started/): The Rust build tool
- [libclang](https://rust-lang.github.io/rust-bindgen/requirements.html):
  Used by `libcdio-sys` for generating Rust bindings to libcdio
- [libcdio](https://github.com/libcdio/libcdio): Consult your package manager

### Build
```sh
cargo build --release
```
Omit `--release` for debug builds.
The build outputs should be in the `target` directory.

### Run the tests
```sh
cargo test
```

To run ignored tests, which require some extra setup:
```sh
cargo test -- --include-ignored
```

### Use the provided Git Hooks
These automatically perform lint and formatting checks:
```sh
git config core.hooksPath .githooks
```

To skip hooks, use `--no-verify`:
```sh
git commit --no-verify
```

### How to use a local build of libcdio
Build libcdio with the `--without-versioned-libs` option.
```sh
cd libcdio
autoreconf -f -i
./configure --without-versioned-libs
make
```

Set the following environment variables:
```sh
# Set this to the (absolute!) path of your libcdio build directory
export LIBCDIO_ROOT="/home/skran/libcdio"

export SYSTEM_DEPS_LIBCDIO_NO_PKG_CONFIG="yes"
export SYSTEM_DEPS_LIBCDIO_SEARCH_NATIVE="$LIBCDIO_ROOT/lib/driver/.libs"
export SYSTEM_DEPS_LIBCDIO_INCLUDE="$LIBCDIO_ROOT/include"
export SYSTEM_DEPS_LIBCDIO_LIB="cdio"

export SYSTEM_DEPS_LIBISO9660_NO_PKG_CONFIG="yes"
export SYSTEM_DEPS_LIBISO9660_SEARCH_NATIVE="$LIBCDIO_ROOT/lib/iso9660/.libs"
export SYSTEM_DEPS_LIBISO9660_INCLUDE="$LIBCDIO_ROOT/include"
export SYSTEM_DEPS_LIBISO9660_LIB="iso9660"

export SYSTEM_DEPS_LIBUDF_NO_PKG_CONFIG="yes"
export SYSTEM_DEPS_LIBUDF_SEARCH_NATIVE="$LIBCDIO_ROOT/lib/udf/.libs"
export SYSTEM_DEPS_LIBUDF_INCLUDE="$LIBCDIO_ROOT/include"
export SYSTEM_DEPS_LIBUDF_LIB="udf"

export BINDGEN_EXTRA_CLANG_ARGS="-I$LIBCDIO_ROOT/include"
export LD_LIBRARY_PATH="$LIBCDIO_ROOT/lib/driver/.libs:$LIBCDIO_ROOT/lib/iso9660/.libs:$LIBCDIO_ROOT/lib/udf/.libs:$LD_LIBRARY_PATH"

# For macOS
export LD_LIBRARY_PATH="$LIBCDIO_ROOT/lib/driver/.libs:$LIBCDIO_ROOT/lib/iso9660/.libs:$LIBCDIO_ROOT/lib/udf/.libs:$DYLD_LIBRARY_PATH"
```

That's it. Cargo should now use your local copy of libcdio.

# License
Copyright (C) 2026 Shiva Kiran Koninty <shiva@skran.xyz>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by the
Free Software Foundation, either version 3 of the License, or (at your
option) any later version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
