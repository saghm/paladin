#!/bin/sh

# Make temporary build directory
mkdir -p paladin

# Build IDE
cargo build --release || exit

# Copy resources into build directory
cp -rv comprehensive_example.pal inconsolata target/release/paladin paladin

# Make zip for distribution
zip -r paladin.zip paladin

# Remove temporary directory
rm -r paladin
