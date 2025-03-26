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

