import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  Keypair,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";

import {
  TOKEN_PROGRAM_ID,
  createMint,
  mintTo,
  AccountLayout,
  createInitializeAccountInstruction,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";

import { AmmCapstone } from "../target/types/amm_capstone";

/* -------------------------------------------------- */
/* Helper: create PDA-owned token vault               */
/* -------------------------------------------------- */
async function createVaultAccount(
  provider: anchor.AnchorProvider,
  mint: PublicKey,
  owner: PublicKey
): Promise<PublicKey> {

  const vault = Keypair.generate();
  const connection = provider.connection;

  const lamports =
    await connection.getMinimumBalanceForRentExemption(
      AccountLayout.span
    );
    

  const tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey: vault.publicKey,
      space: AccountLayout.span,
      lamports,
      programId: TOKEN_PROGRAM_ID,
    }),
    createInitializeAccountInstruction(
      vault.publicKey,
      mint,
      owner
    )
  );

  await sendAndConfirmTransaction(
    connection,
    tx,
    [(provider.wallet as any).payer, vault]
  );

  return vault.publicKey;
}

/* -------------------------------------------------- */
/* TEST SUITE                                         */
/* -------------------------------------------------- */
async function initVaultPda(
  provider: anchor.AnchorProvider,
  vaultPda: PublicKey,
  mint: PublicKey,
  owner: PublicKey,
  programId: PublicKey
) {

  const connection = provider.connection;
  const payer = (provider.wallet as any).payer;

  const lamports =
    await connection.getMinimumBalanceForRentExemption(
      AccountLayout.span
    );

  const tx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: vaultPda,
      space: AccountLayout.span,
      lamports,
      programId: TOKEN_PROGRAM_ID,
    }),
    createInitializeAccountInstruction(
      vaultPda,
      mint,
      owner
    )
  );

  await sendAndConfirmTransaction(
    connection,
    tx,
    [payer]
  );
}

describe("amm-capstone", () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program =
    anchor.workspace.AmmCapstone as Program<AmmCapstone>;

  const wallet = provider.wallet as anchor.Wallet;
  const payer = (wallet as any).payer;

  // Seeds
  const VAULT_A_SEED = Buffer.from("vault_a");
  const VAULT_B_SEED = Buffer.from("vault_b");
  const TREASURY_SEED = Buffer.from("treasury");
  const POOL_SEED = Buffer.from("pool");
  const VAULT_AUTH_SEED = Buffer.from("vault_authority");

  // Accounts
  let treasuryPda: PublicKey;
  let poolPda: PublicKey;
  let vaultAuthorityPda: PublicKey;

  let tokenAMint: PublicKey;
  let tokenBMint: PublicKey;
  let lpMint: PublicKey;

  let userTokenA: PublicKey;
  let userTokenB: PublicKey;
  let userLp: PublicKey;

  let vaultA: PublicKey;
  let vaultB: PublicKey;

  /* -------------------------------------------------- */
  /* GLOBAL SETUP                                       */
  /* -------------------------------------------------- */
    
  before(async () => {

    // ---------- PDAs ----------
    [treasuryPda] = PublicKey.findProgramAddressSync(
      [TREASURY_SEED],
      program.programId
    );

    [vaultAuthorityPda] = PublicKey.findProgramAddressSync(
      [VAULT_AUTH_SEED],
      program.programId
    );

    // ---------- Token Mints ----------
    tokenAMint = await createMint(
      provider.connection,
      payer,
      wallet.publicKey,
      null,
      6
    );

    tokenBMint = await createMint(
      provider.connection,
      payer,
      wallet.publicKey,
      null,
      6
    );

    [poolPda] = PublicKey.findProgramAddressSync(
      [
        POOL_SEED,
        tokenAMint.toBuffer(),
        tokenBMint.toBuffer(),
      ],
      program.programId
    );

    lpMint = await createMint(
      provider.connection,
      payer,
      vaultAuthorityPda,
      null,
      6
    );

    // ---------- USER TOKEN ACCOUNTS ----------
    userTokenA = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        tokenAMint,
        wallet.publicKey
      )
    ).address;

    userTokenB = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        tokenBMint,
        wallet.publicKey
      )
    ).address;

    userLp = (
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        lpMint,
        wallet.publicKey
      )
    ).address;

    

    // ---------- PDA VAULT ACCOUNTS ----------
  // ---------- VAULT TOKEN ACCOUNTS (CORRECT WAY) ----------
vaultA = await createVaultAccount(
  provider,
  tokenAMint,
  vaultAuthorityPda
);

vaultB = await createVaultAccount(
  provider,
  tokenBMint,
  vaultAuthorityPda
);

    // ---------- FUND USER ----------
    await mintTo(
      provider.connection,
      payer,
      tokenAMint,
      userTokenA,
      wallet.publicKey,
      1_000_000_000
    );

    await mintTo(
      provider.connection,
      payer,
      tokenBMint,
      userTokenB,
      wallet.publicKey,
      1_000_000_000
    );
  });
  

  /* -------------------------------------------------- */
  /* TEST 1: INIT TREASURY                              */
  /* -------------------------------------------------- */

  it("Initializes treasury", async () => {

    await program.methods
      .initTreasury()
      .accountsStrict({
        payer: wallet.publicKey,
        treasury: treasuryPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
  });

  /* -------------------------------------------------- */
  /* TEST 2: CREATE POOL                                */
  /* -------------------------------------------------- */

  it("Creates liquidity pool", async () => {

    await program.methods
      .createPool()
      .accountsStrict({
        payer: wallet.publicKey,
        tokenAMint,
        tokenBMint,
        pool: poolPda,
        lpMint,
        vaultAuthority: vaultAuthorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([payer])
      .rpc();
  });

  /* -------------------------------------------------- */
  /* TEST 3: ADD LIQUIDITY                              */
  /* -------------------------------------------------- */

  it("Adds liquidity", async () => {

    await program.methods
      .addLiquidity(
        new anchor.BN(500_000_000),
        new anchor.BN(500_000_000)
      )
      .accountsStrict({
        user: wallet.publicKey,
        pool: poolPda,
        userTokenA,
        userTokenB,
        vaultA,
        vaultB,
        lpMint,
        userLp,
        vaultAuthority: vaultAuthorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
     // ✅ initialize vault token accounts (REQUIRED)
  await initVaultPda(
    provider,
    vaultA,
    tokenAMint,
    vaultAuthorityPda,
    program.programId
  );

  await initVaultPda(
    provider,
    vaultB,
    tokenBMint,
    vaultAuthorityPda,
    program.programId
  );

    const poolAccount =
      await program.account.pool.fetch(poolPda);

    console.log("Reserve A:", poolAccount.reserveA.toString());
    console.log("Reserve B:", poolAccount.reserveB.toString());
  });

});