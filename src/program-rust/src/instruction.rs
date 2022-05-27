use std::convert::TryFrom;
use solana_program::{program_error::ProgramError};
use std::str;

#[derive(Debug)]
pub enum Pool_instruction{
    Add_Liquidity(String,String,f64,f64,String),
    Remove_Liquidity(String,String,f64,f64,f64,String),
    Swap_Tokens(String,String,f64,f64,String)
}

impl Pool_instruction {
       pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
                let (&tag, rest) = input
                        .split_first()
                        .ok_or(ProgramError::InvalidInstructionData)?;


           let (&token0_length, rest1) = rest
               .split_first()
               .ok_or(ProgramError::InvalidInstructionData)?;

           let (&token1_length, rest2) = rest1
               .split_first()
               .ok_or(ProgramError::InvalidInstructionData)?;

           let(token0,rest3)=rest2.split_at(token1_length as usize);
           let(token1,rest4)=rest3.split_at(token1_length as usize);

           let token0_string = str::from_utf8(token0).unwrap();
           let token0_name =(token0_string).to_string();
           let token1_string = str::from_utf8(token1).unwrap();
           let token1_name =(token1_string).to_string();

           match tag {
               0=> {
                   let(token0_amount_slice,rest5)=rest4.split_at(8);
                   let(token1_amount_slice,rest6)=rest5.split_at(8);
                   let token0_amount = f64::from_le_bytes(<[u8; 8]>::try_from(token0_amount_slice).unwrap());
                   let token1_amount = f64::from_le_bytes(<[u8; 8]>::try_from(token1_amount_slice).unwrap());
                   let address_to_string =str::from_utf8(rest6).unwrap();
                   let address_to = (address_to_string).to_string();

                    return Ok(Pool_instruction::Swap_Tokens(token0_name,token1_name,token0_amount,token1_amount,address_to))
               }
               1=> {

                   let(token0_amount_slice,rest5)=rest4.split_at(8);
                   let(token1_amount_slice,rest6)=rest5.split_at(8);
                   let token0_amount = f64::from_le_bytes(<[u8; 8]>::try_from(token0_amount_slice).unwrap());
                   let token1_amount = f64::from_le_bytes(<[u8; 8]>::try_from(token1_amount_slice).unwrap());
                   let address_to_string =str::from_utf8(rest6).unwrap();
                   let address_to = (address_to_string).to_string();

                   return Ok(Pool_instruction::Add_Liquidity(token0_name,token1_name,token0_amount,token1_amount,address_to))
               }
               2 =>{
                   let(withdrawn_tokens_slice,restnew)=rest4.split_at(8);
                   let(token0_amount_slice,rest5)=restnew.split_at(8);
                   let(token1_amount_slice,rest6)=rest5.split_at(8);
                   let withdrawn_tokens = f64::from_le_bytes(<[u8; 8]>::try_from(withdrawn_tokens_slice).unwrap());
                   let token0_amount = f64::from_le_bytes(<[u8; 8]>::try_from(token0_amount_slice).unwrap());
                   let token1_amount = f64::from_le_bytes(<[u8; 8]>::try_from(token1_amount_slice).unwrap());
                   let address_to_string =str::from_utf8(rest6).unwrap();
                   let address_to = (address_to_string).to_string();

                   return Ok(Pool_instruction::Remove_Liquidity(token0_name,token1_name,withdrawn_tokens,token0_amount,token1_amount,address_to))
               }
               _ => return Err(ProgramError::InvalidInstructionData),
           }

                }
    }