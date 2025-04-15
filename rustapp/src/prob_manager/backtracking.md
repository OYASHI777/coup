# Backtracking Algorithm Documentation

## 1. Overview
The purpose of this document is to clearly state how backtracking can be used to determine impossible states for a player at some particular turn.

**Example:**
> This algorithm computes the numerical solution to a partial differential equation using the finite difference method. It is designed for high-performance simulation in fluid dynamics.

## 2. Terminology and Standards
Useful terms that may be helpful.

## 2.1 The State
"The State" if generally used to refer to the state of the information that we know about the game. This is generally represented by a bunch of constraints. These are public constraints, inferred constraints, group constraints and impossible constraints.

These are what they represent,

public constraints: Known dead cards for each player
inferred constraints: Known alive cards for each player that are not revealed, but definitely have, given any starting card distribution for the game played so far
group constraints: Known alive and dead cards belonging to a set of players.
impossible constraints: Known cards or combination of cards that a player definitely cannot have, given any starting card distribution for the game player so far

## 2.1 Cards
Cards are represented as A, B, C, D, E, each corresponding to the 5 possible cards in Coup. We may sometimes use None to represent a null space or unknown card.

## 2.2 Card States
Cards have 2 states, Dead or Alive.

## 2.2 Card Permutations

In general we represent the possible cards players can have with a vector of vector of cards. It looks something like this:

[[A, A], [B, C], [C, D], [B, E], [A, B], [C, E], [D, D, E]]

Each of the players in [0, 5] have 2 card, while player 6 (the centre pile) has 3 cards.

When unknown we may also represent this incompletely, as follows:

[[A], [], [C, D], [E], [A, B], [C, E], [E]]

To differentiate the states of the cards, they are often seperated into 2 vectors.

public_constraints: [[A], [], [C], [E], [A, B], [], []]
inferred_constraints: [[], [], [D], [], [], [C, E], [E]]

public_constraints represent known dead cards, inferred_constraints represent cards that are known but alive and unrevealed.

## 3. Algorithm Description
### 3.1 Overview
The algorithm generally consists of 2 parts. They are loosely named the "forward-pass" and the "backward-pass".

The goal of the forward-pass is to update the current understanding of the 

### 3.1 Discard
### 3.1 Pseudocode
Outline the algorithm steps using pseudocode.

```plaintext
Algorithm: FiniteDifferenceSolver
Input: Initial condition u0, time step Δt, spatial step Δx, total time T
Output: Approximated solution u at time T

1. Initialize u = u0.
2. For time t = 0 to T with step Δt:
    a. Compute spatial derivatives using finite differences.
    b. Update u based on the equation:
       u_new = u + Δt * (D * Laplacian(u) + f(u, x, t))
    c. Set u = u_new.
3. Return u.
