use x86_64::instructions::port::Port;

pub fn reboot() -> ! {
    unsafe {
        let mut port = Port::<u8>::new(0x64);

        // send reset command
        loop {
            // wait until buffer is empty
            while port.read() & 0x02 != 0 {}
            port.write(0xFE);
        }
    }
}