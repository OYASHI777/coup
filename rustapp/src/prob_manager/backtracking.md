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
impossible constraints: Known cards or combination of cards that a player definitely cannot have alive, given any starting card distribution for the game player so far. They may however have it dead.

In general this is mostly concerning information regarding the possible permutation of cards, after some number of moves. It ignores coins, which are trivially trackable.

These are coined "constraints" as they also represent the constraint that are placed on the card permutations because of the moves that have been played. All legal card permutations must satisfy this. Any permutation that fails to fully satisfy cannot be reached. Here are some examples,

public constraints: If a player has discarded a card. It is now dead.
inferred constraints: If a player had 1 card, and they revealed it, and randomly redrew a new card (including the revealed card) from the pile, but later discards a card different from the revealed card, the pile then has to have the revealed card, as the player redrew a different card from the pile.
group constraints: If a player reveals a card and randomly redrew a new card (including the revealed card) from the pile. We know there is at least 1 of the revealed card amongst 1 card from the player and all 3 cards from the pile.
impossible constraints: If all cards of a particular kind have been discarded, no player alive may have this card alive as there are non left.

## 2.1 Cards
Cards are represented as A, B, C, D, E, each corresponding to the 5 possible cards in Coup. We may sometimes use None to represent a null space or unknown card. There are 3 of each card.

## 2.2 Card States
Cards have 2 states, Dead or Alive.

## 2.3 Hand Sizes
All 6 players can have 2 cards, with the pile holding 3 cards.

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

The goal of the forward-pass is to update the current understanding of the game's state based on the latest move possible. This is achieved by updating the state of the game.

### 3.1 Forward-pass
### 3.1 Backward-pass
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
