use std::collections::HashMap;
use cpu::cpu::instruction::Instruction;

//TODO verify if all opcodes are in the correct AddressingMode.

#[derive(Debug)]
pub struct Opcode {
    pub opcode: u8,
    pub cycles: u8,
    pub num_bytes: u8,
}

impl Opcode {
    pub fn new(opcode: u8, cycles: u8, num_bytes: u8) -> Opcode {
        Opcode {
            opcode: opcode,
            cycles: cycles,
            num_bytes: num_bytes,
        }
    }

    // TODO: Verify if these functions can be forced to be inlined with Rust.
    
    // checks if the last 3 bytes of bytes represent a register 8 (A,B,C,D,E,H,L)
    fn is_reg8(bytes: u8) -> bool { (bytes & 0b111) != 0b110  }

    fn is_prefix(opcode: u8) -> bool {
        opcode == 0xCB
    }

    /* 8-Bit Load Group */
    fn is_ld_r_r(opcode: u8) -> bool {
        opcode >> 6 == 0b01 && Opcode::is_reg8(opcode >> 3) && Opcode::is_reg8(opcode)
    }
    fn is_ld_r_n(opcode: u8) -> bool {
        opcode >> 6 == 0b00 && Opcode::is_reg8(opcode >> 3) && !Opcode::is_reg8(opcode)
    }
    fn is_ld_r_hl(opcode: u8) -> bool {
        opcode >> 6 == 0b01 && Opcode::is_reg8(opcode >> 3) && !Opcode::is_reg8(opcode)
    }
    fn is_ld_hl_r(opcode: u8) -> bool {
        opcode >> 6 == 0b01 && !Opcode::is_reg8(opcode >> 3) && Opcode::is_reg8(opcode)
    }
    fn is_ld_hl_n(opcode: u8) -> bool { opcode == 0x36 }
    fn is_ld_a_bc(opcode: u8) -> bool { opcode == 0x0A }
    fn is_ld_a_de(opcode: u8) -> bool { opcode == 0x1A }
    fn is_ld_a_nn(opcode: u8) -> bool { opcode == 0x3A }
    fn is_ld_bc_a(opcode: u8) -> bool { opcode == 0x02 }
    fn is_ld_de_a(opcode: u8) -> bool { opcode == 0x12 }

    /* 16-Bit Load Group */
    fn is_ld_dd_nn(opcode: u8) -> bool {
        opcode >> 6 == 0b00 && (opcode & 0b1) == 0b1
    }
    fn is_ld_hl_nn(opcode: u8) -> bool { opcode == 0x2A }
    fn is_ld_nn_hl(opcode: u8) -> bool { opcode == 0x22 }
    fn is_ld_sp_hl(opcode: u8) -> bool { opcode == 0xF9 }
    fn is_push_qq(opcode: u8) -> bool { 
        opcode >> 6 == 0b11 && (opcode & 0b1111) == 0b0101
    }
    fn is_pop_qq(opcode: u8) -> bool {
        opcode >> 6 == 0b11 && (opcode & 0b1111) == 0b0001
    }

    /* 8-Bit Arithmetic Group */
    fn is_add_a_r(opcode: u8) -> bool {
        opcode >> 3 == 0b10000
    }
    fn is_add_a_n(opcode: u8) -> bool { opcode == 0xC6 }
    fn is_adc_a_r(opcode: u8) -> bool { opcode >> 3 == 0b10001 }
    fn is_adc_a_n(opcode: u8) -> bool { opcode == 0xCE }

    fn is_sub_r(opcode: u8) -> bool { opcode >> 3 == 0b10010 }
    fn is_sub_n(opcode: u8) -> bool { opcode == 0xD6 }

    fn is_sbc_a_r(opcode: u8) -> bool { opcode >> 3 == 0b10011 }
    fn is_sbc_a_n(opcode: u8) -> bool { opcode == 0xDE }

    fn is_and_r(opcode: u8) -> bool { opcode >> 3 == 0b10100 }
    fn is_and_n(opcode: u8) -> bool { opcode == 0xE6 }

    fn is_or_r(opcode: u8) -> bool { opcode >> 3 == 0b10110 }
    fn is_or_n(opcode: u8) -> bool { opcode == 0xF6 }
    
    fn is_xor_r(opcode: u8) -> bool { opcode >> 3 == 0b10101 }
    fn is_xor_n(opcode: u8) -> bool { opcode == 0xEE }

    fn is_cp_r(opcode: u8) -> bool { opcode >> 3 == 0b10111 }
    fn is_cp_n(opcode: u8) -> bool { opcode == 0xFE }

    fn is_inc_s(opcode: u8) -> bool {
        opcode >> 6 == 0b0 && opcode & 0b111 == 0b100
    }
    fn is_dec_s(opcode: u8) -> bool {
        opcode >> 6 == 0b0 && opcode & 0b111 == 0b101
    }
    
    /* General-Purpose Arithmetic and CPU Control Groups */
    fn is_daa(opcode: u8) -> bool { opcode == 0x27 }
    fn is_cpl(opcode: u8) -> bool { opcode == 0x2F }
    fn is_ccf(opcode: u8) -> bool { opcode == 0x3F }
    fn is_scf(opcode: u8) -> bool { opcode == 0x37 }
    fn is_nop(opcode: u8) -> bool { opcode == 0x00 }
    fn is_halt(opcode: u8) -> bool { opcode == 0x76 }
    fn is_di(opcode: u8) -> bool { opcode == 0xF3 }
    fn is_ei(opcode: u8) -> bool { opcode == 0xFB }

    /* 16-Bit Arithmetic Group */
    fn is_add_hl_ss(opcode: u8) -> bool {
        opcode >> 6 == 0b00 && opcode & 0b1111 == 0b1001
    }
    fn is_inc_ss(opcode: u8) -> bool {
        opcode >> 6 == 0b00 && opcode & 0b1111 == 0b0011
    }
    fn is_dec_ss(opcode: u8) -> bool {
        opcode >> 6 == 0b00 && opcode & 0b1111 == 0b1011
    }

    /* Rotate and Shift Group */
    fn is_rlca(opcode: u8) -> bool { opcode == 0x07 }
    fn is_rla(opcode: u8) -> bool { opcode == 0x17 }
    fn is_rrca(opcode: u8) -> bool { opcode == 0x0F }
    fn is_rra(opcode: u8) -> bool { opcode == 0x1F }

    /* Jump Group */
    fn is_jp_nn(opcode: u8) -> bool { opcode == 0xC3 }
    fn is_jp_cc_nn(opcode: u8) -> bool {
        opcode >> 6 == 0b11 && opcode & 0b111 == 0b010
    }
    fn is_jr_e(opcode: u8) -> bool { opcode == 0x18 }
    fn is_jr_c_e(opcode: u8)-> bool { opcode == 0x38 }
    fn is_jr_nc_e(opcode: u8) -> bool { opcode == 0x30 }
    fn is_jr_z_e(opcode: u8) -> bool { opcode == 0x28 }
    fn is_jr_nz_e(opcode: u8) -> bool { opcode == 0x20 }
    fn is_jp_hl(opcode: u8) -> bool { opcode == 0xE9 }

    /* Call and Return Group */
    fn is_call_nn(opcode: u8) -> bool { opcode == 0xCD }
    fn is_call_cc_nn(opcode: u8) -> bool {
        opcode >> 6 == 0b11 && opcode & 0b111 == 0b100
    }
    fn is_ret(opcode: u8) -> bool { opcode == 0xC9 }
    fn is_ret_cc(opcode: u8) -> bool {
        opcode >> 6 == 0b11 && opcode & 0b111 == 0b000
    }
    fn is_rst_p(opcode: u8) -> bool {
        opcode >> 6 == 0b11 && opcode & 0b111 == 0b111
    }
    
    /* GB only */
    fn is_reti(opcode: u8) -> bool { opcode == 0xD9 }
    fn is_add_sp_n(opcode: u8) -> bool { opcode == 0xE8 }
    fn is_ldi_hl_a(opcode: u8) -> bool { opcode == 0x22 }
    fn is_ldd_hl_a(opcode: u8) -> bool { opcode == 0x32 } 
    fn is_ldi_a_hl(opcode: u8) -> bool { opcode == 0x2A }
    fn is_ldd_a_hl(opcode: u8) -> bool { opcode == 0x3A }
    fn is_ldh_n_a(opcode: u8) -> bool { opcode == 0xE0 }
    fn is_ldh_a_n(opcode: u8) -> bool { opcode == 0xF0 }
    fn is_ldhl_sp_n(opcode: u8) -> bool { opcode == 0xF8 }
    fn is_ld_nn_sp(opcode: u8) -> bool { opcode == 0x08 }
    fn is_stop(opcode: u8) -> bool { opcode == 0x10 }
    fn is_swap_n(opcode: u8) -> bool { opcode >> 3 == 0b00110 }

    /* CB-Prefixed */
    fn is_cb_rlc_s(opcode: u8) -> bool { opcode >> 3 == 0b00000 }
    fn is_cb_rrc_s(opcode: u8) -> bool { opcode >> 3 == 0b00001 }
    fn is_cb_rl_s(opcode: u8) -> bool { opcode >> 3 == 0b00010 }
    fn is_cb_rr_s(opcode: u8) -> bool { opcode >> 3 == 0b00011 }
    fn is_cb_sla_s(opcode: u8) -> bool { opcode >> 3 == 0b00100 }
    fn is_cb_sra_s(opcode: u8) -> bool { opcode >> 3 == 0b00101 }
    fn is_cb_srl_s(opcode: u8) -> bool { opcode >> 3 == 0b00111 }
    fn is_cb_bit_s(opcode: u8) -> bool { opcode >> 6 == 0b01 }
    fn is_cb_res_s(opcode: u8) -> bool { opcode >> 6 == 0b10 }
    fn is_cb_set_s(opcode: u8) -> bool { opcode >> 6 == 0b11 }

    //TODO functions like: fn opcode_reg8_lhs opcode_reg8_rhs opcode_reg16_lhs opcode_reg16_rhs
    
    //All opcodes are either 1, 2 or 3 bytes long.
    fn is_2bytes_long(opcode: u8) -> bool {
        Opcode::is_ld_r_n(opcode)     ||  Opcode::is_ld_hl_n(opcode)  ||  Opcode::is_add_a_n(opcode)  ||
        Opcode::is_adc_a_n(opcode)    ||  Opcode::is_sub_n(opcode)    ||  Opcode::is_sbc_a_n(opcode)  ||
        Opcode::is_and_n(opcode)      ||  Opcode::is_or_n(opcode)     ||  Opcode::is_xor_n(opcode)    ||
        Opcode::is_cp_n(opcode)       ||  Opcode::is_jr_e(opcode)     ||  Opcode::is_jr_c_e(opcode)   ||
        Opcode::is_jr_nc_e(opcode)    ||  Opcode::is_jr_z_e(opcode)   ||  Opcode::is_jr_nz_e(opcode)  ||
        Opcode::is_add_sp_n(opcode)   ||  Opcode::is_ldh_n_a(opcode)  ||  Opcode::is_ldh_a_n(opcode)  ||
        Opcode::is_ldhl_sp_n(opcode)  ||  Opcode::is_stop(opcode)
    }
    fn is_3bytes_long(opcode: u8) -> bool {
        Opcode::is_ld_a_nn(opcode)   ||  Opcode::is_ld_dd_nn(opcode)  ||
        Opcode::is_ld_hl_nn(opcode)  ||  Opcode::is_ld_nn_hl(opcode)  ||  Opcode::is_jp_nn(opcode)       ||
        Opcode::is_jp_cc_nn(opcode)  ||  Opcode::is_call_nn(opcode)   ||  Opcode::is_call_cc_nn(opcode)  ||
        Opcode::is_ld_nn_sp(opcode)
    }
}

//TODO: use tuple struct with one element instead.
#[derive(Debug)]
pub struct OpcodeMap {
    map: HashMap<u16, Opcode>,
}

impl OpcodeMap {
    pub fn new() -> OpcodeMap {
        let mut op_map: HashMap<u16, Opcode> = HashMap::new();
        for opcode in 0x0..0xFF {
            let mut num_bytes: u8 = 2;
            if opcode != 0xCB {
                num_bytes = 1;
                if Opcode::is_2bytes_long(opcode) {
                    num_bytes = 2;
                } else if Opcode::is_3bytes_long(opcode) {
                    num_bytes = 3;
                }
            } else {
                num_bytes = 0;
            }
            let opcode_obj: Opcode = Opcode::new(opcode, 0, num_bytes);
            op_map.insert(opcode as u16, opcode_obj);
        }
        OpcodeMap {
            map: op_map,
        }
    }

    pub fn opcode(&self, opcode: u8) -> &Opcode {
        match self.map.get(&(opcode as u16)) {
            Some(opcode_obj) => opcode_obj,
            None => panic!("Non existing opcode: {}", opcode),
        }
    }

    pub fn fetch_instructions(&self, bytes: &Vec<u8>) -> Vec<Instruction> {
        let mut data_iter = bytes.iter();
        let mut all_instructions = Vec::new();
        loop {
            match data_iter.next() {
                Some(opcode_byte) => {
                    let opcode_obj: &Opcode = self.opcode(*opcode_byte);
                    let mut nbytes = opcode_obj.num_bytes;
                    if *opcode_byte == 0xCB {
                        nbytes = 2; //1 for supporting the CB prefix + 1 for the CB-prefixed instruction.
                    }

                    let mut instruction: Instruction = vec![0; nbytes as usize];
                    instruction[0] = *opcode_byte;

                    //starts from 1 because the first byte was already added.
                    for n in 1..nbytes {
                        match data_iter.next() {
                            Some(byte) => {
                                instruction[n as usize] = *byte;
                            },
                            None => panic!("Invalid opcode instruction size."),
                        }
                    }

                    print!("0x");
                    for i in instruction.iter() {
                        print!("{:01$x}", i, 2);
                    }
                    println!("");
                    all_instructions.push(instruction);
                },
                None => break,
            }
        }
        all_instructions
    }
}


