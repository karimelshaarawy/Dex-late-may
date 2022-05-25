use solana_program::{program_error::ProgramError};
#[derive(Debug)]
pub enum Pool_instruction{
    Add_Liquidity,
    Remove_Liquidity,
    Swap_Tokens
}

impl Pool_instruction {
       pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
                let (&tag, rest) = input
                        .split_first()
                        .ok_or(ProgramError::InvalidInstructionData)?;

                Ok(match tag {
                        0 => Pool_instruction::Add_Liquidity ,
                        1 => Pool_instruction::Remove_Liquidity,
                        2 => Pool_instruction::Swap_Tokens,
                        _ => return Err(ProgramError::InvalidInstructionData),
                    })
                }
    }