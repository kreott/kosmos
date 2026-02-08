global start
global boot_info
%define PHYS_OFFSET 0xFFFF800000000000
extern long_mode_start

section .data
align 16

; BootInfo struct
boot_info:
    dq 0                   ; api_version = 0
    dq memory_regions      ; ptr to MemoryRegions array
    dq 1                   ; len = 1
    dq 0                   ; framebuffer = None
    dq PHYS_OFFSET         ; physical_memory_offset
    dq 0                   ; recursive_index = None
    dq 0                   ; rsdp_addr = None
    dq 0                   ; tls_template = None
    dq 0                   ; ramdisk_addr = None
    dq 0                   ; ramdisk_len
    dq 0                   ; kernel_addr
    dq 0                   ; kernel_len
    dq 0                   ; kernel_image_offset
    dq 0                   ; kernel_stack_bottom
    dq 0                   ; kernel_stack_len
    dq 0                   ; _test_sentinel

; FFI-safe MemoryRegions struct
memory_regions:
    dq memory_region_0      ; pointer to first MemoryRegion
    dq 1                    ; number of regions

; MemoryRegion itself
memory_region_0:
    dq 0x100000             ; start = 1 MiB
    dq 0x40000000           ; end = 1 GiB
    dq 0                    ; kind = Usable
    
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
    ; L4 -> L3
    mov eax, page_table_l3       ; physical addr of L3
    or eax, 0b11                 ; present + writable
    mov [page_table_l4], eax     ; first L4 entry points to L3

    ; L3 -> L2
    mov eax, page_table_l2       ; physical addr of L2
    or eax, 0b11                 ; present + writable
    mov [page_table_l3], eax     ; first L3 entry points to L2

    ; L2 -> 2 MiB pages
    mov ecx, 0

.loop_l2:

    mov eax, ecx
    shl eax, 21          ; multiply by 2 MiB
    or eax, 0b10000011   ; present + writable + huge page
    mov [page_table_l2 + ecx*8], eax
    inc ecx
    cmp ecx, 512
    jne .loop_l2

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
    mov word [0xB8004], 0x4F52
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