use crate::factory::Factory;
use crate::liquidity_pool::Liquidity_pool;
use borsh::{BorshDeserialize, BorshSerialize};

use agsol_borsh_schema::BorshSchema;

#[derive(BorshSerialize, BorshDeserialize,BorshSchema)]
pub(crate) struct router{
    factory:Factory
}

impl<'a> router {
    pub(crate) fn add_liquidity(& 'a mut self, token_A:String, token_B:String, amount_A_desired:f64, amount_B_desired:f64, address_to:String)
                                -> (f64,f64,f64){
        let mut key;
        let token1_amount;
        let token2_amount;
            if token_A<token_B {
                key =  [token_A.clone(), token_B.clone()].concat();
                token1_amount=amount_A_desired;
                token2_amount=amount_B_desired;
            }
             else {
                 key = [token_B.clone(), token_A.clone()].concat();
                 token1_amount=amount_B_desired;
                 token2_amount=amount_A_desired;

             }
        if(! self.factory.pair_addresses.contains_key(key.as_str())){
            Factory::create_pair(& mut self.factory,token_A.clone(),token_B.clone(),amount_A_desired,amount_B_desired,address_to);
            (amount_A_desired,amount_B_desired,(amount_A_desired*amount_B_desired).sqrt())
        }
        else {
            let pool: &mut Liquidity_pool = self.factory.pair_addresses.get_mut(key.as_str()).unwrap();
            let (reserveA, reserveB) = pool.get_reserves();

            let ratioB = reserveB / reserveA * token2_amount;
            if ratioB <= token2_amount {
                pool.set_reserves(token1_amount + reserveA, ratioB+reserveB);
                pool.set_kLast((token1_amount + reserveA)*(ratioB+reserveB));
                let pool_tokens = pool.mint(address_to, token1_amount, ratioB);
                (token1_amount, ratioB, pool_tokens)
            } else {
                let ratioA = reserveA / reserveB * token1_amount;
                pool.set_reserves(ratioA + reserveA, token2_amount + reserveB);
                pool.set_kLast((ratioA + reserveA)*(token2_amount + reserveB));
                let pool_tokens = pool.mint(address_to, ratioA, token2_amount);
                (ratioA, token2_amount, pool_tokens)
            }


        }



    }

    pub(crate)  fn remove_liquidity(&mut self,token_A:String,token_B:String,liquidity_withdrawn:f64,amount_A_min:f64,amount_B_min:f64,address_to:String)
        ->(f64,f64){

        let mut key ;
        if token_A<token_B {
            key =  [token_A.clone(), token_B.clone()].concat();
        }
        else {
            key = [token_B.clone(), token_A.clone()].concat();
        }
        let pool:&mut Liquidity_pool= self.factory.pair_addresses.get_mut(key.as_str()).unwrap();
        //make sure that client has enough liquidity
        assert!(*pool.liquidity_providers.get(address_to.as_str()).unwrap()>=liquidity_withdrawn);

        let (reserveA,reserveB)= pool.get_reserves();
        let liquidity_ratio= liquidity_withdrawn/ pool.total_pool_tokens;
        // make sure withdrawn tokens are more than minimum
        let token1_withdrawn=liquidity_ratio*reserveA;
        let token2_withdrawn=liquidity_ratio*reserveB;
        assert!(token1_withdrawn>=amount_A_min && token2_withdrawn>=amount_B_min);
        pool.burn(address_to, liquidity_withdrawn);
        pool.set_reserves(reserveA-token1_withdrawn,reserveB-token2_withdrawn);
        pool.set_kLast((reserveA-token1_withdrawn)*(reserveB-token2_withdrawn));

        (token1_withdrawn,token2_withdrawn)
    }

    pub(crate) fn swap_tokens(& 'a mut self, token_A:String, token_B:String, amount_A_desired:f64, amount_B_desired:f64, address_to:String)
                                -> (f64){

        let mut key ;
        let mut result_token_amount;
        if token_A<token_B {
            key =  [token_A.clone(), token_B.clone()].concat();
            let pool:&mut Liquidity_pool= self.factory.pair_addresses.get_mut(key.as_str()).unwrap();
           result_token_amount= pool.swap(amount_A_desired,amount_B_desired,address_to);
        }
        else {
            key = [token_B.clone(), token_A.clone()].concat();
            let pool:&mut Liquidity_pool= self.factory.pair_addresses.get_mut(key.as_str()).unwrap();
           result_token_amount = pool.swap(amount_B_desired,amount_A_desired,address_to)
        }
        result_token_amount
    }

}