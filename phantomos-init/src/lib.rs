#![no_std]
#![feature(default_alloc_error_handler)]
#![feature(panic_info_message)]

#[cfg(target_arch = "x86_64")]
pub mod x86_64;

pub mod allocator;
pub mod devicetree;
pub mod dynloader;
pub mod elf;
pub mod rawmem;
pub mod util;
pub mod writer;

extern crate alloc;

#[macro_use]
extern crate paste;

use core::arch::global_asm;
use core::fmt::Write;
use core::mem::MaybeUninit;
use elf::{Elf64Dyn, Elf64Rela};
use stivale_boot::v2::StivaleStruct;
use uuid::Uuid;
use writer::TerminalWriter;

mod stivale_setup {
    use stivale_boot::v2::{StivaleHeader, StivaleTerminalHeaderTag};

    #[repr(C, align(16))]
    struct Stack([u8; 4096]);

    static STACK: Stack = Stack([0; 4096]);
    static TERMINAL_HEADER_TAG: StivaleTerminalHeaderTag = StivaleTerminalHeaderTag::new().flags(0);

    #[link_section = ".stivale2hdr"]
    #[no_mangle]
    #[used]
    static STIVALE_HDR: StivaleHeader = StivaleHeader::new()
        .stack(&STACK.0[4095] as *const u8)
        .tags(&TERMINAL_HEADER_TAG as *const StivaleTerminalHeaderTag as *const ());
}

#[cfg(target_arch = "x86_64")]
global_asm!(
    r"

.protected main

.global _start
_start:
    // todo: Setup IDTR
    lea rsi, [DYNAMIC_PTRS+rip]
    lea rdx, [_phantomos_init_start+rip]
    mov [rsi], rdx
    lea rdx, [_DYNAMIC+rip]
    mov [rsi+8], rdx
    lea rsi, [_GLOBAL_OFFSET_TABLE_+rip]
    mov [rsi], rdx
    mov qword ptr [rsi+8],0
    lea rdx, [_plt_lookup_sym+rip]
    mov [rsi+16],rdx
    jmp main
    "
);

static mut TERMINAL: MaybeUninit<TerminalWriter> = MaybeUninit::uninit();

fn term<'a>() -> &'a mut TerminalWriter<'static> {
    // This is actually unsound wrt interrupts, but that's not a problem yet.
    // TODO: do something clever
    unsafe { TERMINAL.assume_init_mut() }
}

mod idt_setup {
    use alloc::boxed::Box;
    use core::arch::asm;
    use core::pin::Pin;

    #[repr(C)]
    struct InterruptDescriptor {
        offset1: u16,
        selector: u16,
        ist: u8,
        type_attributes: u8,
        offset2: u16,
        offset3: u32,
        zero: u32,
    }

    enum InterruptBehavior {
        Print(&'static str),
        PrintCrash(&'static str),
        Return,
    }

    use InterruptBehavior::*;

    macro_rules! generate_idt {
        [$(($name:ident, $type:expr)),*] => ([$({
            paste! {
                #[allow(non_snake_case)]
                fn [<launchpad_ $name>] () {
                    ::core::todo!("This is a todo!() macro to prevent, uh, doing a bad");
                }
                let offset: *mut () = [<launchpad_ $name>] as *mut ();
                $crate::idt_setup::InterruptDescriptor {
                    offset1: (offset as usize & 0xFFFF) as u16,
                    offset2: ((offset as usize >> 16) & 0xFFFF) as u16,
                    offset3: (offset as usize >> 32) as u32,
                    selector: 0,
                    ist: 0,
                    type_attributes: 0,
                    zero: 0,
                }
            }
        }),*])
    }

    #[rustfmt::skip]
    pub unsafe fn register_idt() {
        let idt: [InterruptDescriptor; 0x81] = generate_idt![
            (DE, Panic("Divide Error")), (DB, Print("Debug Exception")), (NMI, Print("NMI")), (BP, Print("Breakpoint")),
            (OF, Panic("Overflow")), (BR, Panic("BOUND Range Exceeded")), (UD, Panic("Undefined Opcode")), (NM, Panic("No Math Coprocessor")),
            (DF, Panic("Double Fault")), (CSO, Panic("Coprocessor Segment Overrun")), (TS, Panic("Invalid TSS")), (NP, Panic("Segment Not Present")),
            (SS, Panic("Stack Segment Fault")), (GP, Panic("General Protection")), (PF, Panic("Page Fault")), (RESERVED15, Print("???")),
            (MF, Panic("Math Fault")), (AC, Panic("Alignment Check")), (MC, Panic("Machine Check")), (XM, Panic("SIMD Exception")),
            (VE, Panic("Virtualization Exception")), (CP, Panic("Control Protection Exception")), (RESERVED16, Print("???")), (RESERVED17, Print("???")),
            (RESERVED18, Panic("???")), (RESERVED19, Panic("???")), (RESERVED1A, Panic("???")), (RESERVED1B, Panic("???")),
            (RESERVED1C, Panic("???")), (RESERVED1D, Panic("???")), (RESERVED1E, Panic("???")), (RESERVED1F, Panic("???")),
            (PHANTOM20, Return), (PHANTOM21, Return), (PHANTOM22, Return), (PHANTOM23, Return),
            (PHANTOM24, Return), (PHANTOM25, Return), (PHANTOM26, Return), (PHANTOM27, Return),
            (PHANTOM28, Return), (PHANTOM29, Return), (PHANTOM2A, Return), (PHANTOM2B, Return),
            (PHANTOM2C, Return), (PHANTOM2D, Return), (PHANTOM2E, Return), (PHANTOM2F, Return),
            (PHANTOM30, Return), (PHANTOM31, Return), (PHANTOM32, Return), (PHANTOM33, Return),
            (PHANTOM34, Return), (PHANTOM35, Return), (PHANTOM36, Return), (PHANTOM37, Return),
            (PHANTOM38, Return), (PHANTOM39, Return), (PHANTOM3A, Return), (PHANTOM3B, Return),
            (PHANTOM3C, Return), (PHANTOM3D, Return), (PHANTOM3E, Return), (PHANTOM3F, Return),
            (PHANTOM40, Return), (PHANTOM41, Return), (PHANTOM42, Return), (PHANTOM43, Return),
            (PHANTOM44, Return), (PHANTOM45, Return), (PHANTOM46, Return), (PHANTOM47, Return),
            (PHANTOM48, Return), (PHANTOM49, Return), (PHANTOM4A, Return), (PHANTOM4B, Return),
            (PHANTOM4C, Return), (PHANTOM4D, Return), (PHANTOM4E, Return), (PHANTOM4F, Return),
            (PHANTOM50, Return), (PHANTOM51, Return), (PHANTOM52, Return), (PHANTOM53, Return),
            (PHANTOM54, Return), (PHANTOM55, Return), (PHANTOM56, Return), (PHANTOM57, Return),
            (PHANTOM58, Return), (PHANTOM59, Return), (PHANTOM5A, Return), (PHANTOM5B, Return),
            (PHANTOM5C, Return), (PHANTOM5D, Return), (PHANTOM5E, Return), (PHANTOM5F, Return),
            (PHANTOM60, Return), (PHANTOM61, Return), (PHANTOM62, Return), (PHANTOM63, Return),
            (PHANTOM64, Return), (PHANTOM65, Return), (PHANTOM66, Return), (PHANTOM67, Return),
            (PHANTOM68, Return), (PHANTOM69, Return), (PHANTOM6A, Return), (PHANTOM6B, Return),
            (PHANTOM6C, Return), (PHANTOM6D, Return), (PHANTOM6E, Return), (PHANTOM6F, Return),
            (PHANTOM70, Return), (PHANTOM71, Return), (PHANTOM72, Return), (PHANTOM73, Return),
            (PHANTOM74, Return), (PHANTOM75, Return), (PHANTOM76, Return), (PHANTOM77, Return),
            (PHANTOM78, Return), (PHANTOM79, Return), (PHANTOM7A, Return), (PHANTOM7B, Return),
            (PHANTOM7C, Return), (PHANTOM7D, Return), (PHANTOM7E, Return), (PHANTOM7F, Return),
            (PHANTOM80, Return)
        ];
        let limit = u16::try_from(idt.len()*core::mem::size_of::<InterruptDescriptor>()).unwrap();
        let idt = Box::new(idt);

        #[repr(C)]
        struct IDTR64{
            base: u64,
            limit: u16
        }

        let idtr = IDTR64{base: Box::leak(idt) as *mut _ as u64,limit};
        
        asm!("lidt [{0}]", in(reg) (&idtr));
    }
}

unsafe fn register_idt() {
    idt_setup::register_idt();
}

#[allow(clippy::empty_loop)]
#[no_mangle]
#[cfg(target_arch = "x86_64")]
unsafe extern "C" fn main(stivale_data: *const StivaleStruct) -> ! {
    use core::arch::asm;

    let stivale_data = &*stivale_data;
    TERMINAL.write(TerminalWriter::new(
        stivale_data.terminal().unwrap_or_else(|| loop {}),
    ));
    writeln!(
        term(),
        "Initializing PhantomOS {}...",
        core::env!("CARGO_PKG_VERSION")
    )
    .unwrap();

    if stivale_data.kernel_slide().is_some() {
        writeln!(term(), "Relocating loader symbols...",).unwrap();
        let mut dynamic = core::ptr::addr_of!(dynloader::_DYNAMIC) as *const Elf64Dyn;

        let mut reltab = core::ptr::null::<Elf64Rela>();
        let mut relsize = 0;

        while (*dynamic).d_tag != 0 {
            if (*dynamic).d_tag == 23 {
                reltab = (*dynamic).d_un as *const Elf64Rela;
            } else if (*dynamic).d_tag == 2 {
                relsize = (*dynamic).d_un;
            }
            dynamic = dynamic.add(1);
        }

        let mut i = 0;
        while i < ((relsize as usize) / core::mem::size_of::<Elf64Rela>()) {
            let rel = reltab.add(1);
            if ((*rel).r_info) & 0xffffff == 7 {
                writeln!(term(), "Applying JUMP_SLOT relocation {}...", i).unwrap();
                dynloader::ldresolve(i as u64, 0);
            }
            i += 1;
        }
        writeln!(term(), "Relocating loader symbols... done",).unwrap();
    }

    writeln!(term(), "Setting up interrupts...").unwrap();
    register_idt();
    writeln!(term(), "Setting up interrupts... done").unwrap();

    let boot_volume = stivale_data
        .boot_volume()
        .expect("could not locate boot volume; bootloader may be unsupported");
    let boot_guid = boot_volume.guid;
    let boot_part_guid = boot_volume.part_guid;

    writeln!(term(), "Boot volume UUID: {}", Uuid::from(boot_guid)).unwrap();
    writeln!(
        term(),
        "Boot volume partition UUID: {}",
        Uuid::from(boot_part_guid)
    )
    .unwrap();

    asm!("ud2");

    loop {}
}

#[panic_handler]
fn handle_panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(s) = info.message() {
        writeln!(term(), "panic @ {}: {:?}", info.location().unwrap(), s).unwrap_or(());
    } else {
        writeln!(term(), "panic @ {} (no message)", info.location().unwrap()).unwrap_or(());
    }
    loop {}
}
