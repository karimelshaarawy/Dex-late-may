use std::collections::HashMap;
use hex_literal::hex;
use sha2::{Sha256, Sha512, Digest, Sha256VarCore};
use sha2::digest::core_api::{CoreWrapper, CtVariableCoreWrapper};
use sha2::digest::generic_array::GenericArray;
use sha2::digest::Output;
use sha2::digest::typenum::bit::{B0, B1};
use generic_array::typenum::U5;

// use serde::{Deserialize,Serialize};
// use serde_json;

use crate::liquidity_pool::Liquidity_pool;
// #[derive(Debug,Deserialize,Serialize,PartialEq)]
pub struct Factory<'a>{
    pub(crate) pair_addresses :HashMap<String,  Liquidity_pool<'a>>,
    fee_to:&'a str,
    fee_to_setter:&'a str,

}

impl<'a> Factory<'a>{
    pub fn create_pair<'b>(factory:&'b mut Factory<'b>, token_A:&'b str, token_B:&'b str,reserve_A:f64,reserve_B:f64,address_to:&'b str) {
        assert!(token_A != token_B,"IDENTICAL_ADDRESSES");
        let mut key;
        let mut result;
        // let value1= &mut *[token_A, token_B].join("\n");
        // let value2=  &mut *[token_B, token_A].join("\n");
        if token_A<token_B {
            key =  [token_A, token_B].concat();
            result=Liquidity_pool::new( key.clone(), token_A, token_B);
            result.set_reserves(reserve_A,reserve_B);
        }
         else {
             key = [token_B, token_A].concat();
             result=Liquidity_pool::new( key.clone(),token_B, token_A);
             result.set_reserves(reserve_B,reserve_A);

         }
            result.set_kLast(reserve_A*reserve_B);
        // assert!(!factory.pair_addresses.contains_key(key.as_str()), "Already exists");
        // let  result2=Liquidity_pool::new(factory, token_A, token_B);
        result.mint(address_to,reserve_A,reserve_B);
        factory.pair_addresses.insert(key.to_owned(), result);
        // // API POST REQUEST
        // let pool_Json = serde_json::to_string(&result).unwrap();
        // let client = reqwest::Client::new();
        // let res = client.post("http://httpbin.org/post")
        //     .body(pool_Json)
        //     .send()
        //     .await?;

    }

    fn set_fee_to (&mut self, fee_to:&'a str){
        self.fee_to= fee_to;
    }

    fn set_fee_to_setter(&mut self, fee_to_setter:&'a str){
        self.fee_to_setter= fee_to_setter;
    }
}
