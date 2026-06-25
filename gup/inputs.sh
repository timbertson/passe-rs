#!bash -eu

gup --always
cd "$(dirname "$2")/.."
ls -1 *.toml *.lock | sort > "$1"
find src \
	-name '*.rs' \
	-o -name '*.toml' \
	-o -name '*.lock' \
	| sort \
	> "$1"

gup --contents "$1"
