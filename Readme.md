# tubetube
This is a self hosted youtube kids clone. It collects media from a folder and serves them in a webpage with a similar UI as in the youtube kids app.

# Setup
- set the absolute base path of the media location in [config.toml](config.toml)
- convert all media to mp4 and generate thumbnails using the [fix_media_container_gen_thumbnails.sh](script/fix_media_container_gen_thumbnails.sh)
- populate the media database using the script [gen_movie_list.py](script/gen_movie_list.py)

Run the server using `cargo r`.