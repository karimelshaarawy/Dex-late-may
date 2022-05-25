use crate::factory::Factory;
use crate::liquidity_pool::Liquidity_pool;
use borsh::{BorshDeserialize, BorshSerialize};

use agsol_borsh_schema::BorshSchema;

#[derive(BorshSerialize, BorshDeserialize,BorshSchema)]
pub(crate) struct router<'a>{
    factory:Factory<'a>
}

impl<'a> router <'a>{
    fn add_liquidity(& 'a mut self ,token_A:&'a str,token_B:&'a str,amount_A_desired:f64,amount_B_desired:f64,amount_A_min:f64,amount_B_min:f64,address_to:&'a str)
        -> (f64,f64,f64){
        let mut key;
        let token1_amount;
        let token2_amount;
            if token_A<token_B {
                key =  [token_A, token_B].concat();
                token1_amount=amount_A_desired;
                token2_amount=amount_B_desired;
            }
             else {
                 key = [token_B, token_A].concat();
                 token1_amount=amount_B_desired;
                 token2_amount=amount_A_desired;

             }
        if(! self.factory.pair_addresses.contains_key(key.as_str())){
            Factory::create_pair(& mut self.factory,token_A,token_B,amount_A_desired,amount_B_desired,address_to);
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

    fn remove_liquidity(&mut self,token_A:&'a str,token_B:&'a str,liquidity_withdrawn:f64,amount_A_min:f64,amount_B_min:f64,address_to:&'a str)
        ->(f64,f64){

        let mut key ;
        if token_A<token_B {
            key =  [token_A, token_B].concat();
        }
        else {
            key = [token_B, token_A].concat();
        }
        let pool:&mut Liquidity_pool= self.factory.pair_addresses.get_mut(key.as_str()).unwrap();
        //make sure that client has enough liquidity
        assert!(*pool.liquidity_providers.get(address_to).unwrap()>=liquidity_withdrawn);

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
}