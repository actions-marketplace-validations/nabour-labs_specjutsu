# Brainstorming

Reference for `sj-explore`. Use it in step 3 (go wide, then narrow) and when presenting the
chosen design.

## Going wide

Generating and judging are separate steps. While you list ideas, write each one down even
if it looks wrong; judging comes after the list is long. Aim for eight or more. If the list
stalls, try these prompts:

- **The obvious one.** What would most people build first? Write it down so it stops
  blocking the others.
- **The smallest one.** What is the least that would make one WHEN/THEN outcome true?
- **Do nothing.** What happens if nothing changes? Sometimes that's the answer, or a
  documentation fix is.
- **Inversion.** How would you make the problem worse? The reverse of each answer is an
  idea.
- **Remove a constraint.** What if it were free, instant, or had no backwards-compatibility
  cost? Then ask which part of that idea survives the constraint coming back.
- **Move it.** Could this live in another layer: the CLI, the config, a skill, CI, the docs?
- **Borrow.** How does another tool the user knows solve it?
- **Split it.** Could two smaller changes each do half?

Ask the user for ideas too, one question at a time. They often have one they haven't said.

## Narrowing

Cut the list in this order, and say why each idea goes:

1. **Principles.** Drop anything that breaks `<spec_root>/principles.md` without a good
   reason.
2. **Outcomes.** Drop anything that makes no WHEN/THEN outcome true.
3. **YAGNI.** Remove every part of the remaining ideas that no WHEN/THEN outcome needs:
   options nobody asked for, extension points for imagined users, config for one value.
   What's left is often smaller than any idea on the list.
4. **Cost.** Of what remains, keep the two or three with the clearest trade-offs to put
   to the user.

Merge ideas when two halves make a better whole.

## Presenting the chosen design

Once the user picks a direction, present the design one section at a time, not all at
once. A section is about 200 to 300 words, for example: the data, the commands or API,
error cases, what changes for existing users. After each section, ask whether it looks
right before writing the next. A wrong assumption caught in section one doesn't get built
into the next four.

Keep each section in WHEN/THEN terms where you can; those lines become scenarios in
`sj-propose`.

## Red flags

- You have one idea and are already writing its design.
- Every idea on the list is a variation of the same approach.
- The recommended option has parts no WHEN/THEN outcome needs.
- You presented the whole design in one message and asked "does this look good?"
