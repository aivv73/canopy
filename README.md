# canopy
vcs inspired by theo ideas
git2 -jj does a lot of things right, so let’s borrow from them. - In the new VCS,
we need the ability to commit .env files. - We need private files in the repo that
only certain people can access. - We also need the ability to hold
a merge so others can’t see it. For security patches—public/private
should apply at the change level, not the entire repository—we’ll borrow from DeltaDB:
1. Stable delta identity
   You can reference not only a change, but also a specific operation within that change.

2. Conversation-linked provenance
   “This piece of code originated from this agent message / human decision.”

3. Live worktree history
   No need to wait for a commit to review, sync, discuss, or roll back.

4. CRDT/replicated worktree
   Multiple people/agents can edit a single working copy in parallel.

5. References anchored to deltas, not line numbers
   Comments, reviews, and agent context don’t get lost when lines move. - We still need to make cloning the repository easier - Well, also
the idea that unfinished features shouldn’t be public, like semi-open source
-We’re getting rid of worktrees altogether -The VCS doesn’t involve a real OS/filesystem—it’s more like “just-bash” -Compatibility with Git isn’t needed; we’d have to rely on it too much otherwise -We’ll
write all of this in Rust to make it trendy and youthful. Name: Canopy. CLI: cnp
In fact, this is an experiment to see how far we can go using agent-based development
