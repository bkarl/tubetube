#!/usr/bin/env python3
import os
import sqlite3
import argparse

MOVIE_EXTS = {'.mp4'}
THUMB_EXTS = {'.jpg', '.jpeg', '.png'}


def generate_db(root_folder: str, db_file: str):
    root_folder = os.path.abspath(root_folder)
    if not os.path.isdir(root_folder):
        raise FileNotFoundError(f"Root folder not found: {root_folder}")

    # Map immediate child folders -> integer type
    entries = sorted(
        name for name in os.listdir(root_folder)
        if os.path.isdir(os.path.join(root_folder, name))
    )
    type_map = {name: idx for idx, name in enumerate(entries)}

    rows = []
    for dirpath, _, filenames in os.walk(root_folder, followlinks=True):
        print(f"Scanning folder: {dirpath}")
        movie_files = [f for f in filenames if os.path.splitext(f)[1].lower() in MOVIE_EXTS]
        print(f"Found {len(movie_files)} files.")
        if not movie_files:
            continue

        # Determine top-level subfolder relative to root_folder
        rel = os.path.relpath(dirpath, root_folder)
        if rel == ".":
            top_level = None
        else:
            top_level = rel.split(os.sep)[0]

        media_type = type_map.get(top_level, -1)  # -1 means unknown / not in a root subfolder

        for movie in movie_files:
            base_name = os.path.splitext(movie)[0]
            movie_path = os.path.abspath(os.path.join(dirpath, movie))

            thumb_found = None
            for ext in THUMB_EXTS:
                thumb_path = os.path.join(dirpath, f"{base_name}{ext}")
                if os.path.isfile(thumb_path):
                    thumb_found = os.path.abspath(thumb_path)
                    break

            rows.append((movie_path, thumb_found, media_type))

    # Overwrite DB each run
    if os.path.exists(db_file):
        os.remove(db_file)

    conn = sqlite3.connect(db_file)
    cur = conn.cursor()
    cur.execute("""
    CREATE TABLE media (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        path TEXT NOT NULL,
        thumbnail TEXT,
        type INTEGER NOT NULL
    );
    """)
    if rows:
        cur.executemany("INSERT INTO media (path, thumbnail, type) VALUES (?, ?, ?);", rows)
    conn.commit()

    inserted = cur.execute("SELECT COUNT(*) FROM media;").fetchone()[0]
    conn.close()
    print(f"Inserted {inserted} entries into {db_file}")
    if type_map:
        print("Type mapping (folder -> type):")
        for name, idx in type_map.items():
            print(f"  {idx}: {name}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Generate sqlite DB of movies with thumbnails.")
    parser.add_argument("root_folder", help="Root folder to search for movies")
    args = parser.parse_args()

    try:
        generate_db(args.root_folder, "movies.db")
    except Exception as e:
        print(f"Error: {e}")
        raise
