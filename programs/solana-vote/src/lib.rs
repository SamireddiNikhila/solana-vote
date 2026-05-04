use anchor_lang::prelude::*;

declare_id!("4qVHFhpSY7eZ54AFG8i4XUA9FA9rLTyLCj2ht4Vgp4jx");

#[program]
pub mod solana_vote {
    use super::*;

    /// Initialize a new poll with a question and duration
    pub fn initialize_poll(
        ctx: Context<InitializePoll>,
        question: String,
        duration_days: u8,
    ) -> Result<()> {
        let poll = &mut ctx.accounts.poll;
        poll.init(
            question,
            duration_days,
            ctx.accounts.authority.key(),
            Clock::get()?.unix_timestamp,
        )
    }

    /// Add a candidate option to the poll
    pub fn add_candidate(
        ctx: Context<AddCandidate>,
        name: String,
    ) -> Result<()> {
        let poll = &mut ctx.accounts.poll;
        require!(poll.is_active()?, PollError::PollNotActive);
        require!(!poll.is_full(), PollError::TooManyCandidates);

        poll.add_candidate(name)?;
        Ok(())
    }

    /// Cast a vote for a candidate
    pub fn cast_vote(
        ctx: Context<CastVote>,
        candidate_index: u8,
    ) -> Result<()> {
        let poll = &mut ctx.accounts.poll;
        let voter_record = &mut ctx.accounts.voter_record;

        require!(poll.is_active()?, PollError::PollNotActive);
        require!(!voter_record.has_voted, PollError::AlreadyVoted);
        require!(
            (candidate_index as usize) < poll.candidates.len(),
            PollError::InvalidCandidate
        );

        poll.cast_vote(candidate_index)?;
        voter_record.record_vote(candidate_index, ctx.accounts.voter.key());

        emit!(VoteCast {
            poll: poll.key(),
            voter: ctx.accounts.voter.key(),
            candidate_index,
        });

        Ok(())
    }

    /// Close the poll early (authority only)
    pub fn close_poll(ctx: Context<ClosePoll>) -> Result<()> {
        let poll = &mut ctx.accounts.poll;
        require!(poll.is_active()?, PollError::PollNotActive);
        poll.closed = true;
        Ok(())
    }
}

// ─────────────────────────────────────────────
//  ACCOUNT STRUCTS
// ─────────────────────────────────────────────

#[account]
pub struct Poll {
    pub authority: Pubkey,       // Who created this poll
    pub question: String,        // The poll question (max 200 chars)
    pub candidates: Vec<Candidate>, // Up to 8 candidates
    pub start_time: i64,         // Unix timestamp
    pub end_time: i64,           // Unix timestamp
    pub closed: bool,            // Manual close flag
    pub bump: u8,
}

impl Poll {
    pub const MAX_QUESTION_LEN: usize = 200;
    pub const MAX_CANDIDATES: usize = 8;

    // Space: discriminator + pubkey + string + vec<candidate> + i64x2 + bool + u8
    pub const SPACE: usize = 8 + 32 + (4 + Self::MAX_QUESTION_LEN)
        + (4 + Self::MAX_CANDIDATES * Candidate::SPACE)
        + 8 + 8 + 1 + 1;

    /// Initialize poll fields
    pub fn init(
        &mut self,
        question: String,
        duration_days: u8,
        authority: Pubkey,
        now: i64,
    ) -> Result<()> {
        require!(
            question.len() <= Self::MAX_QUESTION_LEN,
            PollError::QuestionTooLong
        );
        require!(duration_days > 0 && duration_days <= 30, PollError::InvalidDuration);

        self.authority = authority;
        self.question = question;
        self.candidates = Vec::new();
        self.start_time = now;
        self.end_time = now + (duration_days as i64) * 86_400;
        self.closed = false;
        Ok(())
    }

    /// Check if the poll is still accepting votes
    pub fn is_active(&self) -> Result<bool> {
        let now = Clock::get()?.unix_timestamp;
        Ok(!self.closed && now >= self.start_time && now < self.end_time)
    }

    /// True if the candidate list is at capacity
    pub fn is_full(&self) -> bool {
        self.candidates.len() >= Self::MAX_CANDIDATES
    }

    /// Append a new candidate
    pub fn add_candidate(&mut self, name: String) -> Result<()> {
        require!(name.len() <= Candidate::MAX_NAME_LEN, PollError::NameTooLong);
        self.candidates.push(Candidate { name, votes: 0 });
        Ok(())
    }

    /// Increment vote count for given candidate index
    pub fn cast_vote(&mut self, index: u8) -> Result<()> {
        self.candidates[index as usize].votes = self
            .candidates[index as usize]
            .votes
            .checked_add(1)
            .ok_or(PollError::Overflow)?;
        Ok(())
    }

    /// Returns the leading candidate (or None if no votes yet)
    pub fn winner(&self) -> Option<&Candidate> {
        self.candidates.iter().max_by_key(|c| c.votes)
    }

    /// Total votes cast across all candidates
    pub fn total_votes(&self) -> u64 {
        self.candidates.iter().map(|c| c.votes).sum()
    }
}

// ─────────────────────────────────────────────
//  CANDIDATE  (nested struct, NOT an account)
// ─────────────────────────────────────────────

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Candidate {
    pub name: String,  // Max 50 chars
    pub votes: u64,
}

impl Candidate {
    pub const MAX_NAME_LEN: usize = 50;
    // Space: (4 + 50) + 8
    pub const SPACE: usize = (4 + Self::MAX_NAME_LEN) + 8;
}

// ─────────────────────────────────────────────
//  VOTER RECORD  — one PDA per (poll, voter)
// ─────────────────────────────────────────────

#[account]
pub struct VoterRecord {
    pub voter: Pubkey,
    pub poll: Pubkey,
    pub has_voted: bool,
    pub candidate_index: u8,
    pub voted_at: i64,
    pub bump: u8,
}

impl VoterRecord {
    pub const SPACE: usize = 8 + 32 + 32 + 1 + 1 + 8 + 1;

    pub fn record_vote(&mut self, candidate_index: u8, voter: Pubkey) {
        // Safe to unwrap — Clock::get() in instruction already succeeded
        self.voter = voter;
        self.has_voted = true;
        self.candidate_index = candidate_index;
        self.voted_at = Clock::get().unwrap().unix_timestamp;
    }
}

// ─────────────────────────────────────────────
//  INSTRUCTION CONTEXTS
// ─────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(question: String)]
pub struct InitializePoll<'info> {
    #[account(
        init,
        payer = authority,
        space = Poll::SPACE,
        seeds = [b"poll", authority.key().as_ref(), &question.as_bytes()[..question.len().min(32)]],        bump
    )]
    pub poll: Account<'info, Poll>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddCandidate<'info> {
    #[account(
        mut,
        has_one = authority,
    )]
    pub poll: Account<'info, Poll>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub poll: Account<'info, Poll>,

    #[account(
        init_if_needed,
        payer = voter,
        space = VoterRecord::SPACE,
        seeds = [b"voter", poll.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub voter_record: Account<'info, VoterRecord>,

    #[account(mut)]
    pub voter: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePoll<'info> {
    #[account(mut, has_one = authority)]
    pub poll: Account<'info, Poll>,

    pub authority: Signer<'info>,
}

// ─────────────────────────────────────────────
//  EVENTS
// ─────────────────────────────────────────────

#[event]
pub struct VoteCast {
    pub poll: Pubkey,
    pub voter: Pubkey,
    pub candidate_index: u8,
}

// ─────────────────────────────────────────────
//  ERRORS
// ─────────────────────────────────────────────

#[error_code]
pub enum PollError {
    #[msg("Poll is not currently active")]
    PollNotActive,
    #[msg("You have already voted in this poll")]
    AlreadyVoted,
    #[msg("Candidate index out of range")]
    InvalidCandidate,
    #[msg("Poll has reached max candidates (8)")]
    TooManyCandidates,
    #[msg("Question exceeds 200 characters")]
    QuestionTooLong,
    #[msg("Candidate name exceeds 50 characters")]
    NameTooLong,
    #[msg("Duration must be between 1 and 30 days")]
    InvalidDuration,
    #[msg("Vote count overflow")]
    Overflow,
}