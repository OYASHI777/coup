# Coup The Resistance Bot/AI 🚀  
Yet another *AI*—ugh. It's really just a game theory solver with a neural network to learn how good a particular position is. 


## Description
Coup is an imperfect information social deduction game, that pits players against each other. Just like poker, players do not know other players' cards, but unlike poker, players can swap cards with a central pile to gain more information. This adds a layer of complexity in managing "chance nodes" and player actions that are dependent on it. 
**Game Rules**: [Official Rulebook](https://www.qugs.org/rules/r131357.pdf)

## Features
🟢 **Basic game simulation**  
  - 🟢 Handles full Coup game flow  
  - ⚪ Better game design architecture instead of many branches  
  
🟡 **Card counting mechanism for pruning search**  
  - 🟢 Create brute force tracker for validation  
  - 🟡 Create memoized card counter for quicker querying of impossible states  
  - ⚪ Front-end visualization  

⚪ **State Probability Tracker**  
  - ⚪ Use a GPU Compute Shader to compute probability of a particular card permutation state  

⚪ **Counterfactual Regret Minimization (CFR)**  
  - ⚪ Basic CFR  
  - ⚪ CVFPR  
  - ⚪ CVFPR + RL  

## Technical Overview
Some papers to reference.  
📖**Relevant Papers**  
  📜 [Accelerating Nash Equilibrium Convergence in Monte Carlo Settings Through Counterfactual Value Based Fictitious Play](https://arxiv.org/abs/2309.03084)  
  📜 [Student of Games: A unified learning algorithm for both perfect and imperfect information games](https://arxiv.org/abs/2112.03178)  
  📜 [Combining Deep Reinforcement Learning and Search for Imperfect-Information Games](https://arxiv.org/abs/2007.13544)  
  📜 [Mastering the Game of Stratego with Model-Free Multiagent Reinforcement Learning](https://arxiv.org/abs/2206.15378)  
    - [Sample Implementation](https://github.com/baskuit/R-NaD)  
  📜 [PerfectDou: Dominating DouDizhu with Perfect Information Distillationg](https://arxiv.org/abs/2203.16406)  


