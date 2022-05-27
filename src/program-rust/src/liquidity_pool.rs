use std::collections::HashMap;
use crate::factory::Factory;
use borsh::{BorshDeserialize, BorshSerialize};


#[derive(BorshSerialize, BorshDeserialize)]
pub struct Liquidity_pool {
    key :String,
    token0_address:String,
    token1_address:String,
    reserve0: f64,
    reserve1: f64,
    klast : f64,
   pub(crate) liquidity_providers :HashMap<String,f64>,
   pub(crate) total_pool_tokens : f64

}

impl Liquidity_pool  {
    pub fn new( new_key:String, tokenA:String, tokenB:String) ->Liquidity_pool{
        Liquidity_pool{
            key:new_key,
            token0_address:tokenA,
            token1_address:tokenB,
            reserve0:0.0,
            reserve1:0.0,
            klast:0.0,
            liquidity_providers:HashMap::new(),
            total_pool_tokens:0.0

        }
    }


    pub(crate) fn get_reserves(&self) -> (f64, f64){
        let reserve_0=self.reserve0;
        let reserve_1=self.reserve1;
        (reserve_0,reserve_1)
    }

    pub(crate) fn set_reserves(&mut self,reserveA:f64,reserveB:f64){
        self.reserve0=reserveA;
        self.reserve1=reserveB;
    }

    pub(crate) fn set_kLast(&mut self,k:f64){
        self.klast=k;
    }

    pub(crate) fn mint(&mut self, address_to:String, token0_num:f64, token1_num:f64) -> f64{
        let result =(token0_num*token1_num).sqrt();
        if self.liquidity_providers.contains_key(address_to.as_str()){
            let value =self.liquidity_providers.get(&address_to);
            let new_value= value.unwrap()+result;
            self.liquidity_providers.insert(address_to,new_value);
        }else{
            self.liquidity_providers.insert(address_to,result);
        }
        self.total_pool_tokens=self.total_pool_tokens+result;
        result

    }
    // tuple containing portions from token 0 and token 1
    pub(crate) fn burn (&mut self, address_from:String, pool_tokens:f64) -> (f64, f64){
        let owned_pool_tokens = *self.liquidity_providers.get(address_from.as_str()).unwrap();
        assert!(pool_tokens<owned_pool_tokens,"owned pool tokens are not enough");
        let token_ratio=pool_tokens/self.total_pool_tokens;
        let new_owned_tokens = owned_pool_tokens- pool_tokens;
        self.liquidity_providers.insert(address_from,new_owned_tokens);
        let  (token_0,token_1)=self.get_reserves();
        self.total_pool_tokens-=pool_tokens;
        (token_ratio*token_0,token_ratio*token_1)
    }

    pub(crate) fn swap (&mut self ,token0_in:f64,token1_in:f64,address_to:String)->f64{
        assert!(token0_in> 0.0 || token1_in>0.0, "invalid in-amount ");
        assert!(token0_in== 0.0 || token1_in==0.0, "only one token should be as input");
        let (reserve_0,reserve_1) = self.get_reserves();
        let mut token_out:f64=0.0;
        if(token0_in>0.0){
            let token_after_fees=token0_in*99.7/100.0;
            let token_1_reserves = self.klast/(token_after_fees+reserve_0);
            token_out=reserve_1-token_1_reserves;
            self.reserve0=reserve_0+token_after_fees;
            self.reserve1=token_1_reserves;
        }else{
            let token_after_fees=token1_in*99.7/100.0;
            let token_0_reserves = self.klast/(token_after_fees+reserve_0);
            token_out=reserve_0-token_0_reserves;
            self.reserve1=reserve_1+token_after_fees;
            self.reserve0=token_0_reserves;

        }
        token_out
    }



}