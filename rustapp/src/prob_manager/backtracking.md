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
The first goal of the backward-pass is to generally determine if a particular permutation of cards is possible to be achieved.
The second goal of the backward-pass is to determine if at some past state, a particular permutation of cards was possible to be achieved.

A particular permutation of cards can be as simple as player 1 having cards [A, C] or player 1 and 2 having [A, C] and [B].

The second goal is useful in the forward-pass as a new action may reveal information about past actions, depending on whether a past state for a particular player was impossible. For example,

Player 1 Reveals Card C Redraws None
Player 2 Discards Card C
Player 1 Discards Card C

If at the time of the Reveal, Player 1 could not have had the state [C, C], then they must have redrawn Card C to be able to discard it later, they could not have had that card otherwise.


### 3.1 Forward-pass
### 3.1 Backward-pass
Backward-pass makes use of backtracking.

It starts at the latest move played, and traverses possible path by moving back up the history of actions.

This is to search through possible "trajectories" in which it might have been possible to arrive at the proposed current particular permutation of cards.

Let's say we are interested in the permutation of cards,

[[] [] [A, C] [] [] [] []]

TODO!

#### 3.1.2 Backward-pass Cases
We use the following notation here to represent the state tracked as we move through the tree in reverse.
This represents a change to our stored (temporary) state as we go from time T+1 to time T.
|  Time   |  P1    |
Move   T  :  []    <- Public  
Move   T  :  [C0]  <- Inferred

Move T + 1:  [C0]  <- Public
Move T + 1:  []    <- Inferred

C0 referring to the first possible Card of the 3 total. This is just for visual documentation.
##### Discards
This is the simplest case. It concerns only the player involved in the discard.
Assume: if a player discards, the public constraint should be updated after. Therefore, when we begin our search, we have the latest public constraints, and therefore when we traverse backwards in time, and some player discards it, it will be in public constraints at T + 1.

Discard C
Case A
Move   T  :  []
Move   T  :  [C0]

Move T + 1:  [C0]
Move T + 1:  []

Case B
Move   T  :  [A0]
Move   T  :  [C0]

Move T + 1:  [C0, A0]
Move T + 1:  []

Case C
Move   T  :  [C1]
Move   T  :  [C0]

Move T + 1:  [C0, C1]
Move T + 1:  []
##### RevealRedraw
This is more complex. It concerns only the player revealing the card, and the pile as the player redraws a random card from the pile after placing their revealed card into the pile.

RevealRedraw Reveal C Redraw None 
Case A
Move   T  :  []     []
Move   T  :  [C0]   []

Move T + 1:  []     []
Move T + 1:  []     []

C0 Revealed

Case B
Move   T  :  []     []
Move   T  :  [C0]   []

Move T + 1:  []      []
Move T + 1:  [C0]    []

C0 Revealed

Case C
Move   T  :  []      []
Move   T  :  [C0]    [C1]

Move T + 1:  []      []
Move T + 1:  [C1]    [(C0)]

C0 Revealed C1 Redrawn

Case D
Move   T  :  []      []
Move   T  :  [C0, A0]    []

Move T + 1:  []      []
Move T + 1:  [A0, (C0)]    []

C0 Revealed and C0 redrawn

Case D
Move   T  :  []      []
Move   T  :  [C0, A0]    [C1]

Move T + 1:  []      []
Move T + 1:  [A0, (C1)]    [(C0)]

C0 Revealed and C1 redrawn

Case E
Move   T  :  []      []
Move   T  :  [C0]    [A0]

Move T + 1:  []      []
Move T + 1:  [A0]    [(C0)]

C0 Revealed and A0 redrawn

Case E
Move   T  :  []      []
Move   T  :  [C0, C1]    []

Move T + 1:  []      []
Move T + 1:  [C0, C1]    []

C0 in hand C1 revealed C1 redrawn

Case E
Move   T  :  []      []
Move   T  :  [C0, C1]    [C2]

Move T + 1:  []      []
Move T + 1:  [C0, C2]    [(C1)]

C0 in hand C1 revealed C2 redrawn

Case F
Move   T  :  []      []
Move   T  :  [C0, A0]    []

Move T + 1:  []      []
Move T + 1:  [C0, A0]    []

A0 in hand C0 Revealed C0 redrawn

Case F
Move   T  :  []      []
Move   T  :  [C0, A0]    [C1]

Move T + 1:  []      []
Move T + 1:  [C1, A0]    [(C0)]

A0 in hand C0 Revealed and C1 redrawn

Case G
Move   T  :  []      []
Move   T  :  [C0, C1]    [A0]

Move T + 1:  []      []
Move T + 1:  [C1, A0]    [(C0)]

C0 Revealed and A0 redrawn

Case E
Move   T  :  []      []
Move   T  :  [C0, C1]    [A0]

Move T + 1:  []      []
Move T + 1:  [C0, A0]    [(C1)]

C0 in hand C1 revealed A0 redrawn


Case E
Move   T  :  []      []
Move   T  :  [C0]    [A0]

Move T + 1:  []      []
Move T + 1:  [A0]    [C0]

C0 Revealed and A0 redrawn

Case Illegal
Move   T  :  []      []
Move   T  :  [C0, E0]    [A0]

Move T + 1:  []      []
Move T + 1:  [A0, B0]    [(C0)]

C0 Revealed and A0 redrawn
E0 needs to continue from T to T + 1. This is not a viable move, as player would have 3 cards in T + 1.