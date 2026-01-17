//! ELFローダ

use crate::error::{KernelError, MemoryError, ProcessError, Result};
use crate::mem::user;
use crate::task::{add_process, add_thread, Process, PrivilegeLevel, Thread};
use crate::init;
use x86_64::structures::paging::PageTableFlags;

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const PT_LOAD: u32 = 1;
const PF_X: u32 = 0x1;
const PF_W: u32 = 0x2;

#[repr(C)]
#[derive(Clone, Copy)]
struct Elf64Header {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Elf64Phdr {
    p_type: u32,
    p_flags: u32,
    p_offset: u64,
    p_vaddr: u64,
    p_paddr: u64,
    p_filesz: u64,
    p_memsz: u64,
    p_align: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct LoadedElf {
    pub entry: u64,
    pub stack_top: u64,
    pub stack_bottom: u64,
}

pub fn load_elf(data: &[u8]) -> Result<LoadedElf> {
    let header = parse_header(data)?;
    validate_header(header)?;

    let phoff = header.e_phoff as usize;
    let phentsize = header.e_phentsize as usize;
    let phnum = header.e_phnum as usize;

    for i in 0..phnum {
        let off = phoff + i * phentsize;
        let phdr = read_phdr(data, off)?;
        if phdr.p_type != PT_LOAD {
            continue;
        }

        let filesz = phdr.p_filesz as usize;
        let memsz = phdr.p_memsz as usize;
        if memsz == 0 {
            continue;
        }

        let file_end = phdr.p_offset as usize + filesz;
        if file_end > data.len() {
            return Err(KernelError::Memory(MemoryError::InvalidAddress));
        }

        let mut flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
        if phdr.p_flags & PF_W != 0 {
            flags |= PageTableFlags::WRITABLE;
        }
        if phdr.p_flags & PF_X == 0 {
            flags |= PageTableFlags::NO_EXECUTE;
        }

        user::map_user_range(phdr.p_vaddr, phdr.p_memsz, flags)?;

        unsafe {
            let dst = phdr.p_vaddr as *mut u8;
            let src = data.as_ptr().add(phdr.p_offset as usize);
            core::ptr::copy_nonoverlapping(src, dst, filesz);

            if memsz > filesz {
                core::ptr::write_bytes(dst.add(filesz), 0, memsz - filesz);
            }
        }
    }

    let stack = user::alloc_user_stack(8)?;

    Ok(LoadedElf {
        entry: header.e_entry,
        stack_top: stack.top,
        stack_bottom: stack.bottom,
    })
}

pub fn spawn_service(path: &str, name: &'static str) -> Result<()> {
    let data = init::fs::read(path).ok_or(KernelError::InvalidParam)?;
    let loaded = load_elf(data)?;

    let process = Process::new(name, PrivilegeLevel::Service, None, 1);
    let pid = process.id();

    if add_process(process).is_none() {
        return Err(KernelError::Process(ProcessError::MaxProcessesReached));
    }

    let entry_fn: fn() -> ! = unsafe { core::mem::transmute(loaded.entry) };
    let stack_size = (loaded.stack_top - loaded.stack_bottom) as usize;
    let thread = Thread::new(pid, name, entry_fn, loaded.stack_bottom, stack_size);

    if add_thread(thread).is_none() {
        return Err(KernelError::Process(ProcessError::MaxProcessesReached));
    }

    Ok(())
}

fn parse_header(data: &[u8]) -> Result<Elf64Header> {
    if data.len() < core::mem::size_of::<Elf64Header>() {
        return Err(KernelError::InvalidParam);
    }
    let ptr = data.as_ptr() as *const Elf64Header;
    Ok(unsafe { *ptr })
}

fn validate_header(header: Elf64Header) -> Result<()> {
    if header.e_ident[0..4] != ELF_MAGIC {
        return Err(KernelError::InvalidParam);
    }
    if header.e_ident[4] != 2 || header.e_ident[5] != 1 {
        return Err(KernelError::InvalidParam);
    }
    if header.e_phentsize as usize != core::mem::size_of::<Elf64Phdr>() {
        return Err(KernelError::InvalidParam);
    }
    Ok(())
}

fn read_phdr(data: &[u8], offset: usize) -> Result<Elf64Phdr> {
    if offset + core::mem::size_of::<Elf64Phdr>() > data.len() {
        return Err(KernelError::InvalidParam);
    }
    let ptr = unsafe { data.as_ptr().add(offset) as *const Elf64Phdr };
    Ok(unsafe { *ptr })
}
