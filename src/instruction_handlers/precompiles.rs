use crate::{
    addressing_modes::{Arguments, Destination, Register1, Register2, Source},
    instruction::InstructionResult,
    vm::Heaps,
    Instruction, Predicate, VirtualMachine,
};
use u256::U256;
use zk_evm_abstractions::{
    aux::Timestamp,
    precompiles::{
        ecrecover::ecrecover_function, keccak256::keccak256_rounds_function,
        secp256r1_verify::secp256r1_verify_function, sha256::sha256_rounds_function,
    },
    queries::LogQuery,
    vm::Memory,
};
use zkevm_opcode_defs::{
    system_params::{
        ECRECOVER_INNER_FUNCTION_PRECOMPILE_ADDRESS, KECCAK256_ROUND_FUNCTION_PRECOMPILE_ADDRESS,
        SECP256R1_VERIFY_PRECOMPILE_ADDRESS, SHA256_ROUND_FUNCTION_PRECOMPILE_ADDRESS,
    },
    PrecompileAuxData, PrecompileCallABI,
};

use super::common::instruction_boilerplate_with_panic;

fn precompile_call(vm: &mut VirtualMachine, instruction: *const Instruction) -> InstructionResult {
    instruction_boilerplate_with_panic(vm, instruction, |vm, args| {
        // TODO check that we're in a system call

        // The user gets to decide how much gas to burn
        // This is safe because system contracts are trusted
        let aux_data = PrecompileAuxData::from_u256(Register2::get(args, vm));
        vm.use_gas(aux_data.extra_ergs_cost)?;

        // TODO record extra pubdata cost

        let mut abi = PrecompileCallABI::from_u256(Register1::get(args, vm));
        if abi.memory_page_to_read == 0 {
            abi.memory_page_to_read = vm.current_frame.heap;
        }
        if abi.memory_page_to_write == 0 {
            abi.memory_page_to_write = vm.current_frame.heap;
        }

        let query = LogQuery {
            timestamp: Timestamp(0),
            key: abi.to_u256(),
            tx_number_in_block: Default::default(),
            aux_byte: Default::default(),
            shard_id: Default::default(),
            address: Default::default(),
            read_value: Default::default(),
            written_value: Default::default(),
            rw_flag: Default::default(),
            rollback: Default::default(),
            is_service: Default::default(),
        };

        let address_bytes = vm.current_frame.address.0;
        let address_low = u16::from_le_bytes([address_bytes[19], address_bytes[18]]);
        match address_low {
            KECCAK256_ROUND_FUNCTION_PRECOMPILE_ADDRESS => {
                keccak256_rounds_function::<_, false>(0, query, &mut vm.heaps);
            }
            SHA256_ROUND_FUNCTION_PRECOMPILE_ADDRESS => {
                sha256_rounds_function::<_, false>(0, query, &mut vm.heaps);
            }
            ECRECOVER_INNER_FUNCTION_PRECOMPILE_ADDRESS => {
                ecrecover_function::<_, false>(0, query, &mut vm.heaps);
            }
            SECP256R1_VERIFY_PRECOMPILE_ADDRESS => {
                secp256r1_verify_function::<_, false>(0, query, &mut vm.heaps);
            }
            _ => {
                // A precompile call may be used just to burn gas
            }
        }

        Register1::set(args, vm, 1.into());

        Ok(())
    })
}

impl Memory for Heaps {
    fn execute_partial_query(
        &mut self,
        _monotonic_cycle_counter: u32,
        mut query: zk_evm_abstractions::queries::MemoryQuery,
    ) -> zk_evm_abstractions::queries::MemoryQuery {
        let page = query.location.page.0 as usize;
        let start = query.location.index.0 as usize * 32;
        let range = start..start + 32;
        if query.rw_flag {
            if range.end > self[page].len() {
                self[page].resize(range.end, 0);
            }
            query.value.to_big_endian(&mut self[page][range]);
        } else {
            let mut buffer = [0; 32];
            for (i, page_index) in range.enumerate() {
                if let Some(byte) = self[page].get(page_index) {
                    buffer[i] = *byte;
                }
            }
            query.value = U256::from_big_endian(&buffer);
            query.value_is_pointer = false;
        }
        query
    }

    fn specialized_code_query(
        &mut self,
        _monotonic_cycle_counter: u32,
        _query: zk_evm_abstractions::queries::MemoryQuery,
    ) -> zk_evm_abstractions::queries::MemoryQuery {
        todo!()
    }

    fn read_code_query(
        &self,
        _monotonic_cycle_counter: u32,
        _query: zk_evm_abstractions::queries::MemoryQuery,
    ) -> zk_evm_abstractions::queries::MemoryQuery {
        todo!()
    }
}

impl Instruction {
    pub fn from_precompile_call(
        abi: Register1,
        burn: Register2,
        out: Register1,
        predicate: Predicate,
    ) -> Self {
        Self {
            arguments: Arguments::new(predicate, 6)
                .write_source(&abi)
                .write_source(&burn)
                .write_destination(&out),
            handler: precompile_call,
        }
    }
}
