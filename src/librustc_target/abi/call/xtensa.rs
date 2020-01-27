// reference: https://github.com/espressif/clang-xtensa/commit/6fb488d2553f06029e6611cf81c6efbd45b56e47#diff-aa74ae1e1ab6b7149789237edb78e688R8450

use crate::abi::call::{ArgAbi, FnAbi, Reg, Uniform};

const NUM_ARG_GPR: u64 = 6;
const MAX_ARG_IN_REGS_SIZE: u64 = 4 * 32;
// const MAX_ARG_DIRECT_SIZE: u64 = MAX_ARG_IN_REGS_SIZE;
const MAX_RET_IN_REGS_SIZE: u64 = 2 * 32;

fn classify_ret_ty<Ty>(arg: &mut ArgAbi<'_, Ty>, xlen: u64) {
    // The rules for return and argument types are the same, so defer to
    // classify_arg_ty.
    let mut remaining_gpr = 2;
    let fixed = true;
    classify_arg_ty(arg, xlen, fixed, &mut remaining_gpr);
}

fn classify_arg_ty<Ty>(arg: &mut ArgAbi<'_, Ty>, xlen: u64, fixed: bool, remaining_gpr: &mut u64) {
    assert!(*remaining_gpr <= NUM_ARG_GPR, "Arg GPR tracking underflow");

    let arg_size = arg.layout.size;
    let alignment = arg.layout.details.align.abi;

    // Determine the number of GPRs needed to pass the current argument
    // according to the ABI. 2*XLen-aligned varargs are passed in "aligned"
    // register pairs, so may consume 3 registers.
    let mut required_gpr = 1u64;

    if !fixed && alignment.bits() == 2 * xlen {
        required_gpr = 2 + (*remaining_gpr % 2);
    } else if arg_size.bits() > xlen && arg_size.bits() <= MAX_ARG_IN_REGS_SIZE {
        required_gpr = (arg_size.bits() + xlen - 1) / xlen;
    }

    let mut stack_required = false;
    if required_gpr > *remaining_gpr {
        stack_required = true;
        required_gpr = *remaining_gpr;
    }
    *remaining_gpr -= required_gpr;

    if !arg.layout.is_aggregate() {
        // All integral types are promoted to XLen width, unless passed on the
        // stack.
        if arg_size.bits() < xlen && !stack_required {
            arg.extend_integer_width_to(xlen);
            return;
        }

        return;
    }

    // Aggregates which are <= 4 * 32 will be passed in registers if possible,
    // so coerce to integers.
    if size.bits() as u64 <= MAX_ARG_IN_REGS_SIZE {
        // Use a single XLen int if possible, 2*XLen if 2*XLen alignment is
        // required, and a 2-element XLen array if only XLen alignment is
        // required.
        if arg_size.bits() <= xlen {
            arg.cast_to(Uniform { unit: Reg::i32(), total: arg_size });
            return;
        } else if alignment.bits() == 2 * xlen {
            arg.cast_to(Uniform { unit: Reg::i64(), total: arg_size });
            return;
        } else {
            arg.extend_integer_width_to((arg_size.bits() + xlen - 1) / xlen);
            return;
        }
    }

    arg.make_indirect();
}

pub fn compute_abi_info<Ty>(fty: &mut FnAbi<'_, Ty>, xlen: u64) {
    if !fty.ret.is_ignore() {
        classify_ret_ty(&mut fty.ret, xlen);
    }

    let return_indirect =
        fty.ret.is_indirect() || fty.ret.layout.size.bits() > MAX_RET_IN_REGS_SIZE;

    let mut remaining_gpr = if return_indirect { NUM_ARG_GPR - 1 } else { NUM_ARG_GPR };

    for arg in &mut fty.args {
        if arg.is_ignore() {
            continue;
        }
        let fixed = true;
        classify_arg_ty(arg, xlen, fixed, &mut remaining_gpr);
    }
}
