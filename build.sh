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

if [ -n "$VEX_TOOLS_PATH" ]; then
	export PATH="$VEX_TOOLS_PATH/gcc/bin:$PATH"
fi

case "${1:-build}" in
build)
	cargo build -r
	arm-none-eabi-objcopy -O binary "$TARGET_DIR/$bin_name" "$TARGET_DIR/$bin_name.bin"
	;;
upload)
  "$VEXCOM_BIN" --write "$TARGET_DIR/$bin_name.bin"
  ;;
*)
	echo "usage: $0 [build]" >&2
	exit 1
	;;
esac
