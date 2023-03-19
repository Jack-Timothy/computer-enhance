use std::fs::File;
use std::io::{Read, Error, stdin};
use std::path::PathBuf;

enum Mov {
    RegMemToFromReg,
    ImmToRegMem,
    ImmToReg,
    MemToAcc,
    AccToMem,
    RegMemToSeg,
    SegToRegMem
}

fn main() -> Result<(), Error>{
    let file_bytes = read_input_executable()
        .expect("Failed to read input executable.");

    let mut index = 0;
    while index < file_bytes.len() {
        let opcode = identify_opcode(file_bytes[index])
            .expect("Could not identify opcode.");

        match opcode {
            Mov::RegMemToFromReg => {
                decode_regmem_tofrom_reg(&file_bytes, &mut index)
                    .expect("Failed to decode Reg/Mem to/from Reg.");
            }
            Mov::ImmToRegMem => {
                decode_imm_to_regmem(&file_bytes, &mut index)
                    .expect("Failed to decode Immediate to Reg/Mem.");
            }
            Mov::ImmToReg => {
                decode_imm_to_reg(&file_bytes, &mut index)
                    .expect("Failed to decode Immediate to Register.");
            }
            Mov::MemToAcc => {
                decode_mem_to_acc(&file_bytes, &mut index)
                    .expect("Failed to decode Memory to Accumulator.");
            }
            Mov::AccToMem => {
                decode_acc_to_mem(&file_bytes, &mut index)
                    .expect("Failed to decode Accumulator to Memory.");
            }
            Mov::RegMemToSeg => {
                println!("RegMemToSeg");
                return Ok(());
            }
            Mov::SegToRegMem => {
                println!("SegToRegMem");
                return Ok(());
            }
        }
        index += 1;
    }
    Ok(())
}

fn decode_mem_to_acc(code: &Vec<u8>, index: &mut usize) -> Result<(), String> {
    let address = read_bytes(code, index, true)
        .expect("Failed to read address.");
    println!("mov ax, [{}]", address);
    Ok(())
}

fn decode_acc_to_mem(code: &Vec<u8>, index: &mut usize) -> Result<(), String> {
    let address = read_bytes(code, index, true)
        .expect("Failed to read address.");
    println!("mov [{}], ax", address);
    Ok(())
}

fn read_bytes(code: &Vec<u8>, index: &mut usize, w_bit:bool) -> Result<i16, String> {
    increment_index(code, index)?;
    if w_bit {
        let low_byte = code[*index];
        increment_index(code, index)?;
        let high_byte = code[*index];
        return Ok(i16::from_be_bytes([high_byte, low_byte]));
    }
    Ok((code[*index] as i8) as i16)
}

fn read_data(code: &Vec<u8>, index: &mut usize, w_bit:bool) -> Result<String, String> {
    let num = read_bytes(code, index, w_bit)
        .expect("Failed to read bytes of data.");
    if w_bit {
        return Ok(std::fmt::format(format_args!("word {}", num)))
    }
    Ok(std::fmt::format(format_args!("byte {}", num)))
}

fn read_operand(code:&Vec<u8>, index:&mut usize, mod_field:u8, rm_field:u8, w_bit:bool) -> Result<String, String> {
    match mod_field {
        0 => { // memory mode; no displacement (except special case)
            if rm_field == 6 {
                let address = read_bytes(code, index, true)
                    .expect("Failed to read bytes of special case.");
                return Ok(std::fmt::format(format_args!("[{}]", address)));
            } else {
                let base_equation = get_base_equation(rm_field)
                    .expect("Failed to get base equation.");
                return Ok(std::fmt::format(format_args!("[{}]", base_equation)));
            }
        }
        1..=2 => { // memory mode; 8-bit or 16-bit displacement
            let displacement = read_bytes(code, index, mod_field == 2)
                .expect("Failed to read bytes of 8-bit displacement.");
            return build_equation(rm_field, displacement);
        }
        3 => { // register mode
            return Ok(String::from(get_reg_name(rm_field, w_bit)));
        }
        _ => {
            return Err(String::from("Invalid MOD field."));
        }
    }
}

fn decode_imm_to_regmem(code:&Vec<u8>, index:&mut usize) -> Result<(), String> {
    let w_bit = (code[*index] & 1) != 0;

    increment_index(code, index)?;
    let mod_field = ((code[*index] & 0b11000000) >> 6) & 0b00000011;
    let rm_field = code[*index] & 0b00000111;

    let destination = read_operand(code, index, mod_field, rm_field, w_bit)
        .expect("Failed to read operand.");
    let data = read_data(code, index, w_bit)
        .expect("Failed to read data.");

    println!("mov {}, {}", destination, data);
    Ok(())
}

fn decode_imm_to_reg(code:&Vec<u8>, index:&mut usize) -> Result<(), String> {
    let w_bit: bool = (code[*index] & 0b00001000) != 0;
    let reg_field: u8 = code[*index] & 0b00000111;
    let reg_name = get_reg_name(reg_field, w_bit);
    increment_index(code, index)?;

    let imm_val: i16;
    if w_bit {
        let low_byte: u8 = code[*index];
        increment_index(code, index)?;
        let high_byte: u8 = code[*index];
        imm_val = i16::from_be_bytes([high_byte, low_byte]);
    } else {
        imm_val = i16::from(code[*index] as i8);
    }
    println!("mov {}, {}", reg_name, imm_val);
    Ok(())
}

fn decode_regmem_tofrom_reg(code:&Vec<u8>, index:&mut usize) -> Result<(), String> {
    let d_bit: bool = (code[*index] & 0b00000010) != 0;
    let w_bit: bool = (code[*index] & 1) != 0;

    increment_index(code, index)?;
    let mod_field = ((code[*index] & 0b11000000) >> 6) & 0b00000011;
    let reg_field = (code[*index] & 0b00111000) >> 3;
    let reg_name = get_reg_name(reg_field, w_bit);
    let rm_field = code[*index] & 0b00000111;

    let regmem = read_operand(code, index, mod_field, rm_field, w_bit)
        .expect("Failed to read operand.");
    if d_bit {
        println!("mov {}, {}", reg_name.to_string(), regmem);
    } else {
        println!("mov {}, {}", regmem, reg_name.to_string());
    }
    Ok(())
}

fn build_equation(rm_field:u8, displacement:i16) -> Result<String, String> {
    let base_equation = get_base_equation(rm_field)
            .expect("Failed to get base equation.");

    if displacement > 0 {
        return Ok(std::fmt::format(format_args!("[{} + {}]", base_equation, displacement)))
    }
    if displacement < 0 {
        return Ok(std::fmt::format(format_args!("[{} - {}]", base_equation, displacement*-1)))
    }
    Ok(std::fmt::format(format_args!("[{}]", base_equation)))
}

fn get_base_equation(rm_field: u8) -> Result<&'static str, String> {
    match rm_field {
        0 => Ok("bx + si"),
        1 => Ok("bx + di"),
        2 => Ok("bp + si"),
        3 => Ok("bp + di"),
        4 => Ok("si"),
        5 => Ok("di"),
        6 => Ok("bp"),
        7 => Ok("bx"),
        _ => Err(std::fmt::format(format_args!("{} is invalid R/M field.", rm_field))),
    }
}

fn get_reg_name(reg_num: u8, w_bit: bool) -> &'static str {
    if w_bit {
        match reg_num {
            0 => "ax",
            1 => "cx",
            2 => "dx",
            3 => "bx",
            4 => "sp",
            5 => "bp",
            6 => "si",
            7 => "di",
            _ => unreachable!(),
        }
    } else {
        match reg_num {
            0 => "al",
            1 => "cl",
            2 => "dl",
            3 => "bl",
            4 => "ah",
            5 => "ch",
            6 => "dh",
            7 => "bh",
            _ => unreachable!(),
        }
    }
}

fn identify_opcode(byte:u8) -> Result<Mov, String> {
    if opcode_is_regmem_tofrom_reg(byte) {
        return Ok(Mov::RegMemToFromReg);
    }

    if opcode_is_imm_to_regmem(byte) {
        return Ok(Mov::ImmToRegMem)
    }

    if opcode_is_imm_to_reg(byte) {
        return Ok(Mov::ImmToReg)
    }

    if opcode_is_mem_to_acc(byte) {
        return Ok(Mov::MemToAcc)
    }
    
    if opcode_is_acc_to_mem(byte) {
        return Ok(Mov::AccToMem)
    }
    
    if opcode_is_regmem_to_seg(byte) {
        return Ok(Mov::RegMemToSeg)
    }
    
    if opcode_is_seg_to_regmem(byte) {
        return Ok(Mov::SegToRegMem)
    }

    Err(String::from("Not a valid opcode"))
}

fn opcode_is_regmem_tofrom_reg(byte:u8) -> bool {
    0b10001000 == (byte & 0b11111100)
}

fn opcode_is_imm_to_regmem(byte:u8) -> bool {
    0b11000110 == (byte & 0b11111110)
}

fn opcode_is_imm_to_reg(byte:u8) -> bool {
    0b10110000 == (byte & 0b11110000)
}

fn opcode_is_mem_to_acc(byte:u8) -> bool {
    0b10100000 == (byte & 0b11111110)
}

fn opcode_is_acc_to_mem(byte:u8) -> bool {
    0b10100010 == (byte & 0b11111110)
}

fn opcode_is_regmem_to_seg(byte:u8) -> bool {
    0b1000110 == byte
}

fn opcode_is_seg_to_regmem(byte:u8) -> bool {
    0b10001100 == byte
}

fn increment_index(code:&Vec<u8>, index:&mut usize) -> Result<(), String> {
    *index += 1;
    if *index == code.len() {
        return Err(String::from("Exceeded code length."));
    }
    Ok(())
}

fn read_input_executable() -> Result<Vec<u8>, Error> {
    // get input file name from user
    let mut user_input = String::new();
    println!("Enter the file name:");
    stdin().read_line(&mut user_input)
        .expect("Failed to read line");
    let file_name = user_input.trim();

    // build the proper file path using the given file name
    let mut path = PathBuf::from("C:\\Users\\Jack\\git\\computer-enhance\\part1\\decoding-multiple-instructions-and-suffixes\\input_files\\");
    path.push(file_name);

    // read the file into a byte buffer
    let mut file = File::open(path)?;
    let mut file_buffer = Vec::new();
    file.read_to_end(&mut file_buffer)?;
    Ok(file_buffer)
}