set -xe
trunk build
static-web-server -p 8080 -x -d ./dist --cache-control-headers false -w static-web-server.toml

# http://localhost:8080/#http://localhost:8080/benchy.stl
# http://localhost:8080/#https://files.printables.com/media/prints/2236/stls/14012_b9139bd5-c68b-46a5-ba28-6513f9715d83/3dbenchy.stl