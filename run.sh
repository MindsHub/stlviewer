set -xe
trunk build
static-web-server -p 8080 -x -d ./dist --cache-control-headers false -w static-web-server.toml