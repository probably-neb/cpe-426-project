all: windows linux
	echo "Done!"

binaries:
	mkdir -p ./binaries

linux : target = x86_64-unknown-linux-gnu
linux : binaries
	cargo build
	cp ./target/debug/binary ./binaries/imposters-$(target)

windows : target = x86_64-pc-windows-gnu
windows: binaries
	cross build --target=$(target)
	cp ./target/$(target)/debug/binary.exe ./binaries/imposters-$(target).exe
