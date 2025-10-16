#!/bin/bash
# Convert many container types to .mp4 (skip existing .mp4) then generate thumbnails for all .mp4 files.
# Now validates and resolves BASE_DIR so find will recurse into subfolders reliably.
BASE_DIR="${1:-.}"

# ...existing code...
# ensure base dir exists and resolve to absolute path
if [ ! -d "$BASE_DIR" ]; then
    echo "Error: base directory not found: $BASE_DIR" >&2
    exit 1
fi
BASE_DIR="$(cd "$BASE_DIR" && pwd -P)"

# Helper: convert one file to mp4 (try stream copy, fallback to re-encode)
convert_to_mp4() {
    local in="$1"
    local out="$2"

    echo "Converting: '$in' -> '$out'"
    # Try fast stream copy first
    if ffmpeg -v error -y -i "$in" -c copy "$out"; then
        echo "Converted (stream copy): $out"
        return 0
    fi

    echo "Stream copy failed, re-encoding to H.264/AAC..."
    if ffmpeg -v error -y -i "$in" -c:v libx264 -preset veryfast -crf 23 -c:a aac -b:a 128k "$out"; then
        echo "Converted (re-encoded): $out"
        return 0
    fi

    echo "Error: conversion failed for '$in'"
    return 1
}

# 1) Convert containers (exclude existing .mp4)
# find will recurse through all subfolders of $BASE_DIR
for infile in `find "$BASE_DIR" -type f \( -iname "*.mkv" -o -iname "*.avi" -o -iname "*.mov" -o -iname "*.webm" -o -iname "*.flv" -o -iname "*.m4v" -o -iname "*.mpg" -o -iname "*.mpeg" \)`; do
    # normalize to absolute path to avoid missing leading slash issues
    dir="$(dirname "$infile")"
    base="$(basename "$infile")"
    name="${base%.*}"
    out_mp4="$dir/$name.mp4"

    echo "Processing file: $infile"

    # Skip if output exists
    if [ -f "$out_mp4" ]; then
        echo "MP4 already exists, skipping conversion: $out_mp4"
        continue
    fi

    convert_to_mp4 "$infile" "$out_mp4"
done

# 2) Generate thumbnails for all mp4 files
for mp4_file in `find "$BASE_DIR" -type f -iname "*.mp4"`; do
    dir=$(dirname "$mp4_file")
    base="$(basename "$mp4_file")"
    name="${base%.*}"
    thumb="$dir/$name.jpg"

    echo "Processing MP4 for thumbnail: $mp4_file"

    if [ -f "$thumb" ]; then
        echo "Thumbnail exists, skipping: $thumb"
        continue
    fi

    duration=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$mp4_file")
    if [ -z "$duration" ]; then
        echo "Warning: Could not determine duration of '$mp4_file', skipping thumbnail."
        continue
    fi

    # Compute midpoint in seconds (integer)
    midpoint=$(awk -v d="$duration" 'BEGIN { if (d<=0 || d=="") print -1; else printf "%d", d/2 }')

    if [ "$midpoint" -lt 0 ]; then
        echo "Warning: invalid midpoint for '$mp4_file', skipping."
        continue
    fi

    # Generate thumbnail
    if ffmpeg -v error -y -ss "$midpoint" -i "$mp4_file" -vframes 1 -q:v 2 "$thumb"; then
        echo "Thumbnail created at ${midpoint}s: $thumb"
    else
        echo "Error: Failed to create thumbnail for '$mp4_file'"
    fi
done

echo "All done."
