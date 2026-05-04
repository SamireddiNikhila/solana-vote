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

  const QUESTION = "Which L2 has the best UX in 2025?";
  const DURATION_DAYS = 7;
  let pollPda: PublicKey;

  before(async () => {
    [pollPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("poll"), authority.publicKey.toBuffer(), Buffer.from(QUESTION)],
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
    const voter = anchor.web3.Keypair.generate();
    const sig = await provider.connection.requestAirdrop(voter.publicKey, anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(sig);

    const [voterRecordPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("voter"), pollPda.toBuffer(), voter.publicKey.toBuffer()],
      program.programId
    );

    await program.methods.castVote(1)
      .accounts({ poll: pollPda, voterRecord: voterRecordPda, voter: voter.publicKey, systemProgram: SystemProgram.programId })
      .signers([voter]).rpc();

    const poll = await program.account.poll.fetch(pollPda);
    expect(poll.candidates[1].votes.toNumber()).to.equal(1);
    console.log(`  ✔ Voted for ${poll.candidates[1].name}`);
  });
});