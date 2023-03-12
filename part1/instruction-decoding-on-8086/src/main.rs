use std::fs::File;
use std::io::{Read, Error};

static INPUT_FILES: [&str; 2] = [
    "C:\\Users\\Jack\\git\\computer-enhance\\part1\\instruction-decoding-on-8086\\input_files\\listing_0037_single_register_mov",
    "C:\\Users\\Jack\\git\\computer-enhance\\part1\\instruction-decoding-on-8086\\input_files\\listing_0038_many_register_mov"
];

fn main() -> Result<(), Error> {
    // Open the file
    let mut file = File::open(INPUT_FILES[1])?;

    // Read the file into a buffer
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    println!("bits 16");

    // Disassemble the file
    let mut index = 0;
    while index < buffer.len() {
        let d_bit = (buffer[index] & 2) != 0;
        let w_bit: bool = (buffer[index] & 1) != 0;

        let reg_num = (buffer[index+1] & 0b00111000) >> 3;
        let reg_name = get_reg_name(reg_num, w_bit);

        let rm_num = buffer[index+1] & 0b00000111;
        let rm_name = get_reg_name(rm_num, w_bit);

        let source_reg: &str;
        let destination_reg: &str;
        if d_bit {
            source_reg = rm_name;
            destination_reg = reg_name;
        } else {
            source_reg = reg_name;
            destination_reg = rm_name;
        }

        println!("mov {destination_reg}, {source_reg}");
        index += 2;
    }

    Ok(())
}

// Helper function to get the register name from its number
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
