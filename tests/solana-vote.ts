import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaVote } from "../target/types/solana_vote";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

describe("solana-vote", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.SolanaVote as Program<SolanaVote>;
  const authority = provider.wallet as anchor.Wallet;

  const QUESTION = "Which L2 has the best UX? v3";  const DURATION_DAYS = 7;
  let pollPda: PublicKey;

  before(async () => {
    const questionBytes = Buffer.from(QUESTION).slice(0, 32);
    [pollPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("poll"), authority.publicKey.toBuffer(), questionBytes],
      program.programId
    );
  });

  it("Initializes a poll", async () => {
    await program.methods
      .initializePoll(QUESTION, DURATION_DAYS)
      .accounts({ poll: pollPda, authority: authority.publicKey, systemProgram: SystemProgram.programId })
      .rpc();
    const poll = await program.account.poll.fetch(pollPda);
    expect(poll.question).to.equal(QUESTION);
    console.log(`  ✔ Poll created: "${poll.question}"`);
  });

  it("Adds candidates", async () => {
    for (const name of ["Arbitrum", "Base", "zkSync", "Optimism"]) {
      await program.methods.addCandidate(name)
        .accounts({ poll: pollPda, authority: authority.publicKey })
        .rpc();
    }
    const poll = await program.account.poll.fetch(pollPda);
    expect(poll.candidates).to.have.length(4);
    console.log(`  ✔ Added ${poll.candidates.length} candidates`);
  });

  it("Casts a vote", async () => {
    // Use authority wallet as voter instead of generating new keypair
    // (avoids devnet airdrop rate limits)
    const [voterRecordPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("voter"), pollPda.toBuffer(), authority.publicKey.toBuffer()],
      program.programId
    );

    await program.methods.castVote(1)
      .accounts({
        poll: pollPda,
        voterRecord: voterRecordPda,
        voter: authority.publicKey,
        systemProgram: SystemProgram.programId
      })
      .rpc();

    const poll = await program.account.poll.fetch(pollPda);
    expect(poll.candidates[1].votes.toNumber()).to.equal(1);
    console.log(`  ✔ Voted for ${poll.candidates[1].name}`);
  });
});