all:
	# cleanup
	rm -rf build/*
	rm -f grub/kosmos.iso
	rm -f target/x86_64-kosmos/release/libkosmos.a
	rm -rf target/x86_64-kosmos/release/libkosmos.d
	
	# make directories
	mkdir -p build/iso build/boot build/rust

	# assemble assembly
	nasm -f elf64 src/asm/boot/header.asm -o build/boot/header.o
	nasm -f elf64 src/asm/boot/main.asm -o build/boot/main.o
	nasm -f elf64 src/asm/boot/main64.asm -o build/boot/main64.o

	# build kernel
	cargo build --target x86_64-kosmos.json --release

	# link
	ld.lld -T linker.ld build/boot/header.o build/boot/main.o build/boot/main64.o --whole-archive target/x86_64-kosmos/release/libkosmos.a --no-whole-archive -o grub/boot/kernel.bin

	# make grub rescue iso
	grub-mkrescue -o build/iso/kosmos.iso grub


run: all
	qemu-system-x86_64 -cdrom build/iso/kosmos.iso

clean: 
	rm -rf target
	rm -rf build



