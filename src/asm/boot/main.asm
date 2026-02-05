global start
extern long_mode_start

section .text
[BITS 32]
start:
    mov esp, stack_top

    call check_multiboot
    call check_cpuid
    call check_long_mode

    call setup_page_tables
    call enable_paging

    lgdt [gdt64.pointer]
    jmp gdt64.code_segment:long_mode_start

    hlt


check_multiboot:
    cmp eax, 0x36D76289 ; magic numbah :O
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, 77 ; M for multiboot error
    mov byte [0xB8008], al
    jmp error

check_cpuid:
    pushfd
    pop eax
    mov ecx, eax
    xor eax, 1 << 21
    push eax
    popfd
    pushfd
    pop eax
    push ecx
    popfd
    cmp eax, ecx
    je .no_cpuid
    ret
.no_cpuid:
    mov al, 12 ; C for cpuid error
    mov byte [0xB8008], al
    jmp error

check_long_mode:
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode

    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29
    jz .no_long_mode

    ret
.no_long_mode:
    mov al, 76 ; L for long mode error
    mov byte [0xB8008], al
    jmp error

; setup paging
setup_page_tables:
    mov eax, page_table_l3
    or eax, 0b11 ; present, writable
    mov [page_table_l4], eax

    mov eax, page_table_l2
    or eax, 0b11 ; present, writable
    mov [page_table_l3], eax

    mov ecx, 0 ; counter
.loop:

    mov eax, 0x200000 ; 2MiB
    mul ecx
    or eax, 0b10000011 ; present, writable, huge page
    mov [page_table_l2 + ecx * 8], eax

    inc ecx ; increment counter
    cmp ecx, 512 ; checks if whole table is mapped
    jne .loop ; if not, continue

    ret

enable_paging:
    ; pass page table location to cpu
    mov eax, page_table_l4
    mov cr3, eax

    ; enable physical address extension, necessary for 64-bit paging
    mov eax, cr4
    or eax, 1 << 5 ; bit 5 is the PAE flag
    mov cr4, eax

    ; enable long mode
    mov ecx, 0xC0000080 ; magic value again :O
    rdmsr
    or eax, 1 << 8 ; bit 8 is the long mode flag
    wrmsr ; write to module specific register

    ; enable paging
    mov eax, cr0
    or eax, 1 << 31 ; bit 31, enable paging flag
    mov cr0, eax

    ret

error:
    ; print "ERR: X" where X is the error code
    mov dword [0xB8000], 0x4F524F45
    mov byte [0xB8004], 0x4F52
    hlt

section .bss
align 4096
page_table_l4:
    resb 4096
page_table_l3:
    resb 4096
page_table_l2:
    resb 4096
stack_bottom:
    resb 4096 * 4 ; defines a 4 kilobyte stack
stack_top:



section .rodata 
gdt64:
    dq 0 ; zero entry
.code_segment: equ $ - gdt64
    dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53) ; code segment
.pointer:
    dw $ - gdt64 - 1
    dq gdt64