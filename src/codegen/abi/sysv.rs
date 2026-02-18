use super::{Abi, Reg, Reg::*};

#[derive(Default)]
pub struct SysVAbi {}

static SYSV_PARAMS: [Reg; 6] = [Rdi, Rsi, Rdx, Rcx, R8, R9];

impl Abi for SysVAbi {
    fn int_arg_regs(&self) -> &'static [Reg] {
        &SYSV_PARAMS
    }
    fn ret_reg(&self) -> Reg {
        Rax
    }
    fn stack_align(&self) -> usize {
        16
    }
    fn shadow_space_size(&self) -> usize {
        0
    }

    fn stack_param_base(&self) -> usize {
        16
    }
}
