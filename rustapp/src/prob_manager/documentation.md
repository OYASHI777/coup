# COUP Card Counter Documentation

## PATH DEPENDENT CASES 

### Knowledge inferred about card redrawn in past RevealRedraw or ExchangeChoice
Sometimes when a player Discards or RevealRedraws, it provides information that leads us to understand that they redrew a particular card in the past.

Example:
- Player A Discards CAPTAIN, which we know he could only have gotten from redrawin when he last RevealRedraw

Conditions:
- [A] Latest move = Player Discard and player has 1 life after / RevealRedraw 2 lives after
    - Discard (1 lives after) => Possible Condition of interest
    - RevealRedraw (2 lives after) => Possible Condition of Interest?
        - (DISCUSSION) But does this matter? Cos player will mix with pile anyways... so even if you
    - Discard (0 lives after) => If player has no more lives after, it will be handled naturally as per the dead case
    - RevealRedraw (1 lives after) => If player RevealRedraws last card, it will be handled naturally as that is their only card
- Card Revealed could only have been obtained from RevealRedraw/Ambassador
    - [B] Player only has access to card via network somehow
    - We know all the unique cards, player could only have gotten the card during RR?

Investigation into Condition [B]

Example 1:
P1 RR DUKE
P2 RR DUKE
P4 RR CAPTAIN
P5 RR DUK
P4 D DUK

There were 2 Dukes among P1, P2 and Pile before P4 RevealRedraw.
P5 had the last Duke in their hand.
All the DUKE was outside of P4 before his RevealRedraw <==> P4 did not have DUKE prior to RevealRedraw 
(Certain) P4 could only have obtained DUKE from their RevealRedraw

Example 2:
P1 RR DUKE
P2 RR DUKE
P4 RR CAPTAIN
P5 D DUK
P4 D DUK

There were 2 Dukes among P1, P2 and Pile before P4 RevealRedraw.
P5 had the last Duke in their hand.
All the DUKE was outside of P4 before his RevealRedraw <==> P4 did not have DUKE prior to RevealRedraw 
(Certain) P4 could only have obtained DUKE from their RevealRedraw
(Certain Effect) Can update and change groups with P4 Single Flag 1 to indicate flag = 0 and single_card_flag = 0

Example 3:
P1 RR DUKE
P2 D DUKE
P3 D DUKE
P4 RR CAPTAIN
P4 RR DUKE

But since its the last DUKE, we now know DUKE is certainly with P4 or PILE.
This should Prune other groups and allow us to know P1 cannot have the DUKE.
(Handled elsewhere) P4 could only have obtained DUKE from RevealRedraw.

Example 4:
P1 RR DUKE
P2 RR DUKE
P4 RR CAPTAIN
P5 RR DUKE
P4 RR DUKE

There were 2 Dukes among P1, P2 and Pile before P4 RevealRedraw.
P5 had the last Duke in their hand.
All the DUKE was outside of P4 before his RevealRedraw <==> P4 did not have DUKE prior to RevealRedraw 
(Certain) P4 could only have obtained DUKE from their RevealRedraw
(Uncertain Effect) 


Example 1:
P1 RR DUK
P2 RR CAP
P2 RR DUK

(Uncertain) P2 DUK could be same DUK that P1 RevealRedraw earlier

Example 2:
P1 RR DUK
P2 AMB
P2 RR DUK

(Uncertain) P2 DUK could be same DUK that P1 RevealRedraw earlier

Example 3:

Effect:
modify groups?
modify impossible?
unmix groups since player did not withdrew a particular card
maybe unmix groups since if the player withdrew their own card, other players could not have drawn their card
- This might need to redo computation from that node?
do example:
if player redrew card, then the revealredraw group for that player would not have existed
another player revealredraw for that card should not mix with that player, can I just like delete the groups?

(DRAFT)
aren't cases where player Reveal last card already handled naturally?

    (PROBLEM FORMULATION)
its about knowing what was redrawn (and as a result not redrawn) in a previous move.
<=> its about knowing that a player could not possibly have a card naturally other than withdrawing
<=> all of that card are outside of player or in pile (player draws it) or all other cards are outside of the player (player redraws it)
=> [CONDITION] RR/D card C card C is the third card known (others Dead or inferred).
    - AND Checking if any previous RR card can be known as a result
        - Some RR/Single card AMB (maybe double)
        - Will this be the same player as Discard or RR?
            - Could it be Player A RR R_0 and redrew C, then Player B RR R_1 and redrew R_0
                - both of which are inferred in one go?
=> [EFFECT] 
    - if redrawn Card C == RevealRedraw Card R was redrawn, 
        === Effectively nothing should change as the player returns to original state ===
        - Added group (containing R) of Player + Pile should only have player
        - Groups card != C => unmixing all card groups where Card != C
            - Flag for Player should not have been set to 1
        - Groups card == C => unmixing card groups 
            - Groups with Player flag 0 will have been set to 1 making it looser. (not favourable for greedy fix)
            - Groups with Player flag 1 will have been unaffected (except by inferred constraint pruning)
    - if redrawn Card C != RevealRedraw Card R was redrawn,
        === Effectively Player has Card C and Pile has Card R ===
        - Added group (containing R) of Player + Pile should only have pile
        - Groups card != C => unmixing all card groups where Card != C
            - Flag for player should not have been set to 1 (as Player did not receive group card)
        - Groups card == C => unmixing card groups 
            - Groups with Player flag 0 will have been set to 1 making it looser. (not favourable for greedy fix)
            - Groups with Player flag 1 will have been unaffected (except by inferred constraint pruning)
=> [RECALCULATION]
    - path dependent solution would have to:
        1. save the redrawn card
        2. recalculate all relevant moves (Discard, RevealRedraw, ExchangeDraw) after RevealRedraw
            - RevealRedraw has to save realised card
            - ExchangeDraw has to save realised cards
            - Perhaps this could be if private information is legit
        3. Save new state
    - performance impact (very rough)
        1. This roughly is run 1/250 games at a later node
        2. It reruns the entire game in a traversal (or it could store a save)
            ~ 20 Actions
        3. Let E[T] be the average time to process an game that does not have moves that need recalculation
            With recalculation,
            E[TR] = 249/250 * E[T] + 1/250 * E[R]
                  = 249/250 * E[T] + 1/250 * (E[T] + E[TR])
            E[TR] = E[T] + 1/250 E[TR]
                  = 250/249 * E[T]
                  = 1.004
            (E[T] + E[TR]) represents processing all nodes until the point of recalculation + the cost of recalculation
        4. We expect around 0.4% increase in processing time on average. E[R] ~= 2.004 E[T]
        5. But of course, its possible that each recursive history traversal increases the probability more and more.
             
=> [ISSUE]
    1. Change the history store
        - Consider if it should be split by player
            - Multiple Vec may overallocate capacity
        - Consider if it should be store historically
            - Less capacity overallocation
            - filter to get a particular player
        - Store Discards too
        - RevealRedraw and Ambassador need to store possible private information
        - [PROBLEM] What is self.revealed_status[player_id].swap_remove for, well things break without it
        - [PROBLEM] in is_part_of_network prev_redraw_counter does not make sense
        - Change to enum of RR and AMB
    2. Add method to evaluate if Card is known for a RevealRedraw
    3. Add method to evaluate if a Card is known for Ambassador
    4. Add method to evaluate if both Cards are known for Ambassador
    5. Consider how this might be recursive?
        - Recalculating history
        - Leads to further history recalculation
        - Note the usually the move counter increases every evaluation, so would need to refactor that
    6. Arguments of functions may need to be changed
        - RevealRedraw with private/partial information
            - Redraw same card => Inferred constraint
            - Redraw different card => mix then double inference or just double inference?
                - mix then double inference has to know that the card inferred is necessarily from the swap
                    - 1 card groups => mixed then deleted effectively
                    - 2 card groups? (look through revealredraw implementation)
                        - Player flag 0 => mixed then might be single flag pruned?
                        - Player flag 1 => mixed?
                    - 3 card groups? (look through revealredraw implementation)
                        - Player flag 0 => mixed then might be single flag pruned?
                        - Player flag 1 => mixed?
        - Ambassador with private/partial information

CATALOG

terminology:
    mixing - when player 0 reveal redraws, and a group_constraint with DIFFERENT_CARD will have flags [0 0 1 0 0 0 1] => [1 0 1 0 0 0 1]
    discovery - when we discover inferred constraint for a particular player

(lives)

full_test_replay_13.log - P1 D DUK INFERRED PILE CAP with PILE DUK
    - P1 RR CAP could only have been DUK => CAP was in PILE
    - CAP being in pile after RR continues unaffected to present move which leads to discovery
full_test_replay_14.log - P2 D CON INFERRED PILE CAP with PILE DUK
    - P4 RR DUK could only have been DUK (no mixing of a RR card group by player before P2)
    - dead player removes flag which leads to discovery
whole_replay_0.log - P5 D DUK INFERRED P2 CON with PILE CON
    - P2 RR CAP could only have redrawn CAP
    - P2 could not have redrawn DUK (no mixing of a RR card group by player before P2)
    - group subset spaces left inference leads to discovery

