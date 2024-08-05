use crate::{Instruction, State};

pub fn debug_instr(
    vm: &State,
    instruction: *const Instruction,
    i: &mut u64,
    only_instr: bool,
    print_registers: bool,
) -> Result<(), ()> {
    *i += 1;

    unsafe {
        println!("{} - INSTR: {:?}", i, (*instruction).variant);
        if !only_instr {
            println!("PC: {}", vm.current_frame.pc_to_u16(instruction));
            println!("GAS LEFT: {}", vm.current_frame.gas);
            println!("SP: {}", vm.current_frame.sp);
        }
    };

    if print_registers {
        for i in 0..16 {
            let reg = vm.registers[i];
            let is_ptr = vm.register_pointer_flags & (1 << i) != 0;
            println!("REG: {} VAL: {} IS_PTR {}", i, reg, is_ptr);
        }
    }

    // if *i == 2689 {
    //     dbg!(&(*vm.current_frame.stack));
    // }

    Ok(())
}
