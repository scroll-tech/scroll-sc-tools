#!/bin/bash

# Create directory to store KZG setup params.
params_dir="$HOME/.openvm/params"
mkdir -p "$params_dir"

# Degrees relevant to Scroll Verifier.
degrees=("22" "24")

# Download setup params for each degree.
for degree in "${degrees[@]}"; do
    url="https://circuit-release.s3.us-west-2.amazonaws.com/scroll-zkvm/params/kzg_bn254_${degree}.srs"
    wget -O "$params_dir/kzg_bn254_${degree}.srs" "$url"
    echo "Downloaded params for degree: $degree"
done
