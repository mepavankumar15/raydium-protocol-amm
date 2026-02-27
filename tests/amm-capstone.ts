import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";

import {
  TOKEN_PROGRAM_ID,
  createMint,
  mintTo,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";

import { AmmCapstone } from "../target/types/amm_capstone"; 
import { expect } from "chai";

describe("amm-capstone", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AmmCapstone as Program<AmmCapstone>;

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

  // These are now PDAs, not Keypairs
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

    // Derive Pool PDA
    [poolPda] = PublicKey.findProgramAddressSync(
      [POOL_SEED, tokenAMint.toBuffer(), tokenBMint.toBuffer()],
      program.programId
    );

    lpMint = await createMint(
      provider.connection,
      payer,
      vaultAuthorityPda,
      null,
      6
    );

    // ---------- DERIVE VAULT PDAs (FIXED) ----------
    // We do NOT create these manually anymore. We derive the address
    // that the program expects to initialize.
    
    [vaultA] = PublicKey.findProgramAddressSync(
      [
        POOL_SEED, 
        tokenAMint.toBuffer(), 
        tokenBMint.toBuffer(), 
        VAULT_A_SEED
      ],
      program.programId
    );

    [vaultB] = PublicKey.findProgramAddressSync(
      [
        POOL_SEED, 
        tokenAMint.toBuffer(), 
        tokenBMint.toBuffer(), 
        VAULT_B_SEED
      ],
      program.programId
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
    // We pass vaultA/vaultB here so the program can initialize them via CPI/Context
    await program.methods
      .createPool()
      .accountsStrict({
        payer: wallet.publicKey,
        tokenAMint,
        tokenBMint,
        pool: poolPda,
        vaultA: vaultA, // Derived PDA
        vaultB: vaultB, // Derived PDA
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
    const amountA = new anchor.BN(500_000_000);
    const amountB = new anchor.BN(500_000_000);

    await program.methods
      .addLiquidity(amountA, amountB)
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

    // Verify state
    const poolAccount = await program.account.pool.fetch(poolPda);
    console.log("Reserve A:", poolAccount.reserveA.toString());
    console.log("Reserve B:", poolAccount.reserveB.toString());
  });

  /* -------------------------------------------------- */
/* TEST 4: SWAP TOKEN A -> TOKEN B                    */
/* -------------------------------------------------- */

it("Swaps token A → token B", async () => {

  // -----------------------------
  // Fetch pool BEFORE swap
  // -----------------------------
  const poolBefore = await program.account.pool.fetch(poolPda);

  const reserveABefore = poolBefore.reserveA.toNumber();
  const reserveBBefore = poolBefore.reserveB.toNumber();

  console.log("Before Swap:");
  console.log("Reserve A:", reserveABefore);
  console.log("Reserve B:", reserveBBefore);

  // -----------------------------
  // User balances BEFORE
  // -----------------------------
  const userABefore =
    Number(
      (await provider.connection.getTokenAccountBalance(userTokenA))
        .value.amount
    );

  const userBBefore =
    Number(
      (await provider.connection.getTokenAccountBalance(userTokenB))
        .value.amount
    );

  // -----------------------------
  // Swap params
  // -----------------------------
  const amountIn = new anchor.BN(100_000_000); // 0.1 token
  const minOut = new anchor.BN(1); // minimal slippage guard

  await program.methods
    .swap(amountIn, minOut)
    .accountsStrict({
      user: wallet.publicKey,
      pool: poolPda,
      userInput: userTokenA,
      userOutput: userTokenB,
      vaultA,
      vaultB,
      vaultAuthority: vaultAuthorityPda,
      treasury: treasuryPda,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .rpc();

  // -----------------------------
  // Fetch pool AFTER swap
  // -----------------------------
  const poolAfter = await program.account.pool.fetch(poolPda);

  const reserveAAfter = poolAfter.reserveA.toNumber();
  const reserveBAfter = poolAfter.reserveB.toNumber();

  console.log("After Swap:");
  console.log("Reserve A:", reserveAAfter);
  console.log("Reserve B:", reserveBAfter);

  // -----------------------------
  // User balances AFTER
  // -----------------------------
  const userAAfter =
    Number(
      (await provider.connection.getTokenAccountBalance(userTokenA))
        .value.amount
    );

  const userBAfter =
    Number(
      (await provider.connection.getTokenAccountBalance(userTokenB))
        .value.amount
    );

  // -----------------------------
  // Assertions
  // -----------------------------

  // User spent token A
  expect(userAAfter).to.be.lessThan(userABefore);

  // User received token B
  expect(userBAfter).to.be.greaterThan(userBBefore);

  // Pool reserve A increased
  expect(reserveAAfter).to.be.greaterThan(reserveABefore);

  // Pool reserve B decreased
  expect(reserveBAfter).to.be.lessThan(reserveBBefore);

  // Constant product invariant check
  const kBefore = reserveABefore * reserveBBefore;
  const kAfter = reserveAAfter * reserveBAfter;

  expect(kAfter).to.be.greaterThanOrEqual(kBefore);
  });

  /* -------------------------------------------------- */
/* TEST 5: SLIPPAGE PROTECTION                        */
/* -------------------------------------------------- */

it("Fails swap when slippage exceeds limit", async () => {

  const amountIn = new anchor.BN(10_000_000); // small input

  // Impossible minimum output (forces failure)
  const impossibleMinOut = new anchor.BN(1_000_000_000);

  let failed = false;

  try {
    await program.methods
      .swap(amountIn, impossibleMinOut)
      .accountsStrict({
        user: wallet.publicKey,
        pool: poolPda,
        userInput: userTokenA,
        userOutput: userTokenB,
        vaultA,
        vaultB,
        vaultAuthority: vaultAuthorityPda,
        treasury: treasuryPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
  } catch (err) {
    console.log("Expected slippage failure:", err.toString());
    failed = true;
  }

  // The transaction MUST fail
  expect(failed).to.equal(true);
  });
});