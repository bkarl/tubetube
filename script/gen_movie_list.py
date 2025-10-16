import os
import json

def find_movies_with_thumbnails(root_folder, output_file):
    movie_extensions = {'.mp4'}
    thumbnail_extensions = {'.jpg', '.jpeg', '.png'}
    results = []

    for dirpath, _, filenames in os.walk(root_folder):
        movie_files = [f for f in filenames if os.path.splitext(f)[1].lower() in movie_extensions]

        for movie in movie_files:
            base_name = os.path.splitext(movie)[0]
            movie_path = os.path.join(dirpath, movie)

            for ext in thumbnail_extensions:
                thumb_name = f"{base_name}{ext}"
                thumb_path = os.path.join(dirpath, thumb_name)

                if os.path.isfile(thumb_path):
                    results.append({
                        "movie": os.path.abspath(movie_path),
                        "thumbnail": os.path.abspath(thumb_path)
                    })
                    break  # Use first matching thumbnail

    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(results, f, indent=2)

if __name__ == "__main__":
    import sys

    if len(sys.argv) != 3:
        print("Usage: python find_movies_json.py <root_folder> <output_file.json>")
    else:
        find_movies_with_thumbnails(sys.argv[1], sys.argv[2])
