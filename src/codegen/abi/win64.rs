use super::{Abi, Reg, Reg::*};

#[derive(Default)]
pub struct Win64Abi {}

static WIN64_PARAMS: [Reg; 4] = [Rcx, Rdx, R8, R9];

impl Abi for Win64Abi {
    fn int_arg_regs(&self) -> &'static [Reg] {
        &WIN64_PARAMS
    }
    fn ret_reg(&self) -> Reg {
        Rax
    }
    fn stack_align(&self) -> usize {
        16
    }
    fn shadow_space_size(&self) -> usize {
        32
    }

    fn stack_param_base(&self) -> usize{
        48
    }
}
