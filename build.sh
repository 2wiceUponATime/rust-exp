#!/bin/sh
set -e

TARGET_DIR=target/thumbv7em-none-eabihf/release

root=$(realpath $(dirname "$0"))
bin_name="${root##*/}"

if [ -f "$root/.env" ]; then
	set -a
	. "$root/.env"
	set +a
fi

if [ -n "$BIN_NAME" ]; then
  bin_name="$BIN_NAME"
fi

case "$1" in
build)
	cargo build -r
	arm-none-eabi-objcopy -O binary "$TARGET_DIR/$bin_name" "$TARGET_DIR/$bin_name.bin"
	;;
upload)
  shift
  "$VEXCOM_BIN" --write "$TARGET_DIR/$bin_name.bin" "$@"
  ;;
*)
	echo "usage: $0 {build,upload}" >&2
	exit 1
	;;
esac
