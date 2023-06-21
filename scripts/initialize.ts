
import * as anchor from '@project-serum/anchor';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { Commitment, ConnectionConfig } from '@solana/web3.js';
import { getOrCreateAssociatedTokenAccount } from '@solana/spl-token'
import DEV_KEY from '../devnet.json';
import { IDL } from '../target/types/staking_nft_reward';
const { PublicKey, Keypair, Connection, SystemProgram } = anchor.web3;

const DEV_ENV = {
  CLUSTER_API: 'https://api.devnet.solana.com',
  PROGRAM_ID: 'BL3gV368of9wpkyG4p5LkpoNK6QaxhuxxV3CYzqrW6T',
  REWARD_TOKEN: 'GnBw4qZs3maF2d5ziQmGzquQFnGV33NUcEujTQ3CbzP3',
  ADMIN: DEV_KEY 
};

const ENV = DEV_ENV;
const VAULT_SEEDS = 'vault';

(async () => {

  const seed = Uint8Array.from(ENV.ADMIN.slice(0, 32));
  const UPDATE_AUTHORITY = Keypair.fromSeed(seed);

  
  const connection = new Connection(ENV.CLUSTER_API, {
    skipPreflight: true,
    preflightCommitment: 'confirmed' as Commitment,
  } as ConnectionConfig );

  const provider = new anchor.AnchorProvider(connection, new NodeWallet(UPDATE_AUTHORITY), {
    skipPreflight: true,
    preflightCommitment: 'confirmed' as Commitment,
  } as ConnectionConfig);
  const program = new anchor.Program(IDL, new PublicKey(ENV.PROGRAM_ID), provider);

  let [vaultPDA, _nonce] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from(VAULT_SEEDS)],
    program.programId
  );

  const result = await program.rpc.initialize(
    {
    accounts: {
      vault: vaultPDA,
      admin: provider.wallet.publicKey, // Admin wallet
      systemProgram: SystemProgram.programId
    }
  })
  console.log('result', result);
  console.log('vault', vaultPDA.toString());
  const rewardAta = await getOrCreateAssociatedTokenAccount(
    connection, 
    UPDATE_AUTHORITY, 
    new PublicKey(ENV.REWARD_TOKEN),
    vaultPDA,
    true
  );
  console.log('rewardAta', rewardAta);
})()