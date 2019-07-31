plugins_dir=$(egrep '^miner_plugin_dir' grin-miner.toml | awk '{ print $NF }' | xargs echo)
if [ -z "$plugins_dir" ]; then
	plugins_dir="target/release/plugins"
fi
mkdir -p "$plugins_dir";

# Install ocl_cuckatoo
cd ocl_cuckatoo
cargo build --release
cd ..
if [ "$(uname)" = "Darwin" ]; then
	cp target/release/libocl_cuckatoo.dylib $plugins_dir/ocl_cuckatoo.cuckooplugin
else
	cp target/release/libocl_cuckatoo.so $plugins_dir/ocl_cuckatoo.cuckooplugin
fi
