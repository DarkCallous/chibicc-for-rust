pub mod sysv;
pub mod win64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reg {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rsi,
    Rdi,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    Rbp,
    Rsp,
}

impl Reg {
    pub fn asm(&self) -> &'static str {
        match self {
            Reg::Rax => "rax",
            Reg::Rbx => "rbx",
            Reg::Rcx => "rcx",
            Reg::Rdx => "rdx",
            Reg::Rsi => "rsi",
            Reg::Rdi => "rdi",
            Reg::R8 => "r8",
            Reg::R9 => "r9",
            Reg::R10 => "r10",
            Reg::R11 => "r11",
            Reg::R12 => "r12",
            Reg::R13 => "r13",
            Reg::R14 => "r14",
            Reg::R15 => "r15",
            Reg::Rbp => "rbp",
            Reg::Rsp => "rsp",
        }
    }
}

pub trait Abi {
    fn int_arg_regs(&self) -> &'static [Reg];
    fn ret_reg(&self) -> Reg;
    fn stack_align(&self) -> usize;
    fn shadow_space_size(&self) -> usize;
    fn stack_param_base(&self) -> usize;
}
