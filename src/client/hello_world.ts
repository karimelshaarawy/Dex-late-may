/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */

import {
  Keypair,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import fs from 'mz/fs';
import path from 'path';
import * as borsh from 'borsh';

import * as BufferLayout from '@solana/buffer-layout';
import { Buffer } from 'buffer';

import {getPayer, getRpcUrl, createKeypairFromFile} from './utils';
import * as buffer from "buffer";

/**
 * Connection to the network
 */
let connection: Connection;

/**
 * Keypair associated to the fees' payer
 */
let payer: Keypair;

/**
 * Hello world's program id
 */
let programId: PublicKey;

/**
 * The public key of the account we are saying hello to
 */
let greetedPubkey: PublicKey;

/**
 * Path to program files
 */
const PROGRAM_PATH = path.resolve(__dirname, '../../dist/program');

/**
 * Path to program shared object file which should be deployed on chain.
 * This file is created when running either:
 *   - `npm run build:program-c`
 *   - `npm run build:program-rust`
 */
const PROGRAM_SO_PATH = path.join(PROGRAM_PATH, 'helloworld.so');

/**
 * Path to the keypair of the deployed program.
 * This file is created when running `solana program deploy dist/program/helloworld.so`
 */
const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, 'helloworld-keypair.json');

// Liquidity Pool class
export class Liquidity_pool {
  key="";
  token0_address="";
  token1_address="";
  reserve0=0;
  reserve1=0;
  klast=0;
  liquidity_providers: Record<string, number>={};
  total_pool_tokens=0
  constructor(fields:{key: string; token0_address: string; token1_address: string; }|undefined=undefined) {
    if(fields){
      this.key=fields.key;
      this.token0_address=fields.token0_address;
      this.token1_address=fields.token1_address;
    }
  }

}

export class Factory
{ pair_addresses: Record<string, Liquidity_pool>={};
  fee_to=""
  fee_to_setter=""
 constructor(fields:{pair_addresses: Record<string, Liquidity_pool>,fee_to:string,fee_to_setter:string}|undefined=undefined) {
    if(fields){
      this.pair_addresses=fields.pair_addresses;
      this.fee_to=fields.fee_to;
      this.fee_to_setter=fields.fee_to_setter;
    }
 }
}

export class router {
  factory: Factory;
  constructor(fields:{factory:Factory}) {
    this.factory=fields.factory;
  }

}

const routerSchema = new Map<any,any> ([
    [
        router,{kind:'struct',fields:[
        ['factory',Factory]
      ]}
    ],
    [
        Factory,{kind:'struct', fields:[
            ['pair_addresses',{kind:'map',key:'string',value:Liquidity_pool}],
            ['fee_to','string'],
            ['fee_to_setter','string']
          ]
        }
      ],
  [  Liquidity_pool,{kind:'struct',fields:[
      ['key','string'],
        ['token0_address','string'],
        ['token1_address','string'],
        ['reserve0','f64'],
        ['reserve1','f64'],
        ['klast','f64'],
        ['liquidity_providers',{kind:'map',key:'string',value:'f64'}],
        ['total_pool_tokens','f64']
    ]}

  ]
]);




/**
 * The state of a greeting account managed by the hello world program
 */
class GreetingAccount {
  counter = 0;
  constructor(fields: {counter: number} | undefined = undefined) {
    if (fields) {
      this.counter = fields.counter;
    }
  }
}

/**
 * Borsh schema definition for greeting accounts
 */
const GreetingSchema = new Map([
  [GreetingAccount, {kind: 'struct', fields: [['counter', 'u32']]}],
]);

/**
 * The expected size of each greeting account.
 */
const GREETING_SIZE = borsh.serialize(
  routerSchema,
    new router({factory:new Factory({pair_addresses:{},fee_to:'',fee_to_setter:''})}), ).length

/**
 * Establish a connection to the cluster
 */
export async function establishConnection(): Promise<void> {
  const rpcUrl = await getRpcUrl();
  connection = new Connection(rpcUrl, 'confirmed');
  const version = await connection.getVersion();
  console.log('Connection to cluster established:', rpcUrl, version);
}

/**
 * Establish an account to pay for everything
 */
export async function establishPayer(): Promise<void> {
  let fees = 0;
  if (!payer) {
    const {feeCalculator} = await connection.getRecentBlockhash();

    // Calculate the cost to fund the greeter account
    fees += await connection.getMinimumBalanceForRentExemption(GREETING_SIZE);

    // Calculate the cost of sending transactions
    fees += feeCalculator.lamportsPerSignature * 100; // wag

    payer = await getPayer();
  }

  let lamports = await connection.getBalance(payer.publicKey);
  if (lamports < fees) {
    // If current balance is not enough to pay for fees, request an airdrop
    const sig = await connection.requestAirdrop(
      payer.publicKey,
      fees - lamports,
    );
    await connection.confirmTransaction(sig);
    lamports = await connection.getBalance(payer.publicKey);
  }

  console.log(
    'Using account',
    payer.publicKey.toBase58(),
    'containing',
    lamports / LAMPORTS_PER_SOL,
    'SOL to pay for fees',
  );
}

/**
 * Check if the hello world BPF program has been deployed
 */
export async function checkProgram(): Promise<void> {
  // Read program id from keypair file
  try {
    const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
    programId = programKeypair.publicKey;
  } catch (err) {
    const errMsg = (err as Error).message;
    throw new Error(
      `Failed to read program keypair at '${PROGRAM_KEYPAIR_PATH}' due to error: ${errMsg}. Program may need to be deployed with \`solana program deploy dist/program/helloworld.so\``,
    );
  }

  // Check if the program has been deployed
  const programInfo = await connection.getAccountInfo(programId);
  if (programInfo === null) {
    if (fs.existsSync(PROGRAM_SO_PATH)) {
      throw new Error(
        'Program needs to be deployed with `solana program deploy dist/program/helloworld.so`',
      );
    } else {
      throw new Error('Program needs to be built and deployed');
    }
  } else if (!programInfo.executable) {
    throw new Error(`Program is not executable`);
  }
  console.log(`Using program ${programId.toBase58()}`);

  // Derive the address (public key) of a greeting account from the program so that it's easy to find later.
  const GREETING_SEED = 'hello';
  greetedPubkey = await PublicKey.createWithSeed(
    payer.publicKey,
    GREETING_SEED,
    programId,
  );

  // Check if the greeting account has already been created
  const greetedAccount = await connection.getAccountInfo(greetedPubkey);
  if (greetedAccount === null) {
    console.log(
      'Creating account',
      greetedPubkey.toBase58(),
      'to say hello to',
    );
    const lamports = await connection.getMinimumBalanceForRentExemption(
      GREETING_SIZE,
    );

    const transaction = new Transaction().add(
      SystemProgram.createAccountWithSeed({
        fromPubkey: payer.publicKey,
        basePubkey: payer.publicKey,
        seed: GREETING_SEED,
        newAccountPubkey: greetedPubkey,
        lamports,
        space: GREETING_SIZE,
        programId,
      }),
    );
    await sendAndConfirmTransaction(connection, transaction, [payer]);
  }
}

function createAddLiquidityInstructionData ():Buffer{
  const dataLayout= BufferLayout.struct([
      BufferLayout.u8('instruction'),
      BufferLayout.u8('token0_length'),
      BufferLayout.u8('token1_length'),
      BufferLayout.cstr('token0_name '),
      BufferLayout.cstr('token1_name '),
      BufferLayout.f64('token0_amount'),
      BufferLayout.f64('token1_amount'),
      BufferLayout.cstr('address_to')
  ]);

  const data =Buffer.alloc(dataLayout.span);
  dataLayout.encode({instruction:1, token0_length:6,token1_length:7,token0_name:'solana',token1_name:'bitcoin', token0_amount:65, token1_amount:33,address_to:'karim'},data);
  return data;
}


function createRemoveLiquidityInstructionData ():Buffer{
  const dataLayout= BufferLayout.struct([
    BufferLayout.u8('instruction'),
    BufferLayout.u8('token0_length'),
    BufferLayout.u8('token1_length'),
    BufferLayout.cstr('token0_name '),
    BufferLayout.cstr('token1_name '),
    BufferLayout.f64('withdrawn_pool_tokens'),
    BufferLayout.f64('token0_amount'),
    BufferLayout.f64('token1_amount'),
    BufferLayout.cstr('address_to')
  ]);

  const data =Buffer.alloc(dataLayout.span);
  dataLayout.encode({instruction:2, token0_length:6,token1_length:7,token0_name:'solana',token1_name:'bitcoin',withdrawn_pool_tokens:10, token0_amount:3, token1_amount:2,address_to:'karim'},data);
  return data;
}

function createSwapInstructionData ():Buffer{
  const dataLayout= BufferLayout.struct([
    BufferLayout.u8('instruction'),
    BufferLayout.u8('token0_length'),
    BufferLayout.u8('token1_length'),
    BufferLayout.cstr('token0_name '),
    BufferLayout.cstr('token1_name '),
    BufferLayout.f64('token0_amount'),
    BufferLayout.f64('token1_amount'),
    BufferLayout.cstr('address_to')
  ]);

  const data =Buffer.alloc(dataLayout.span);
  dataLayout.encode({instruction:0, token0_length:6,token1_length:7,token0_name:'solana',token1_name:'bitcoin', token0_amount:3, token1_amount:0,address_to:'karim'},data);
  return data;
}
/**
 * Say hello
 */
export async function sayHello(): Promise<void> {
  console.log('Saying hello to', greetedPubkey.toBase58());
  const instruction = new TransactionInstruction({
    keys: [{pubkey: greetedPubkey, isSigner: false, isWritable: true}],
    programId,
    data: Buffer.alloc(0), // All instructions are hellos
  });
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [payer],
  );
}

export async function swapTokens(): Promise<void> {
  const instruction = new TransactionInstruction({
    keys: [{pubkey: greetedPubkey, isSigner: false, isWritable: true}],
    programId,
    data: createSwapInstructionData(), // All instructions are hellos
  });
  await sendAndConfirmTransaction(
      connection,
      new Transaction().add(instruction),
      [payer],
  );
}

export async function removeLiquidity(): Promise<void> {
  const instruction = new TransactionInstruction({
    keys: [{pubkey: greetedPubkey, isSigner: false, isWritable: true}],
    programId,
    data: createRemoveLiquidityInstructionData(), // All instructions are hellos
  });
  await sendAndConfirmTransaction(
      connection,
      new Transaction().add(instruction),
      [payer],
  );
}

export async function addLiquidity(): Promise<void> {
  const instruction = new TransactionInstruction({
    keys: [{pubkey: greetedPubkey, isSigner: false, isWritable: true}],
    programId,
    data: createAddLiquidityInstructionData(), // All instructions are hellos
  });
  await sendAndConfirmTransaction(
      connection,
      new Transaction().add(instruction),
      [payer],
  );
}

/**
 * Report the number of times the greeted account has been said hello to
 */
export async function reportGreetings(): Promise<void> {
  const accountInfo = await connection.getAccountInfo(greetedPubkey);
  if (accountInfo === null) {
    throw 'Error: cannot find the greeted account';
  }
  const greeting = borsh.deserialize(
    GreetingSchema,
    GreetingAccount,
    accountInfo.data,
  );
  console.log(
    greetedPubkey.toBase58(),
    'has been greeted',
    greeting.counter,
    'time(s)',
  );
}
