global long_mode_start
extern _start
extern boot_info


section .text
[BITS 64]
long_mode_start:
    ; load null into all data segment registers

    mov rsp, stack_top_64

    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    mov rdi, boot_info   ; use boot_info as first argument to _start
    call _start

.halt:
    hlt
    jmp .halt

section .bss
align 16
stack_bottom: resb 4096 * 4 ; 16 KiB stack
stack_top_64: 