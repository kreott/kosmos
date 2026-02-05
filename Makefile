all:
	rm -rf build/*
	rm -f iso/kosmos.iso
	rm -f target/x86_64-kosmos/release/libkosmos.a
	rm -rf target/x86_64-kosmos/release/libkosmos.d
	mkdir -p build/iso
	mkdir -p build/boot
	mkdir -p build/kernel

	nasm -f elf64 src/asm/boot/header.asm -o build/boot/header.o
	nasm -f elf64 src/asm/boot/main.asm -o build/boot/main.o
	nasm -f elf64 src/asm/boot/main64.asm -o build/boot/main64.o

	cargo build --target x86_64-kosmos.json --release

	objcopy -I elf64-x86-64 -O elf64-x86-64 --strip-all target/x86_64-kosmos/release/libkosmos.a build/kernel/kernel.o


	ld.lld -T linker.ld build/boot/header.o build/boot/main.o build/boot/main64.o --whole-archive target/x86_64-kosmos/release/libkosmos.a --no-whole-archive -o iso/boot/kernel.bin

	grub-mkrescue -o build/iso/kosmos.iso iso


run: all
	qemu-system-x86_64 -cdrom build/iso/kosmos.iso

clean: 
	rm -rf target
	rm -rf build
	rm -f iso/kosmos.iso