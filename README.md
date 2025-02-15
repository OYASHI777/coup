# Coup The Resistance Bot 🚀

## Description
An attempt to create a bot that plays Coup, by employing a modified version of Counterfactual Regret Minimization. 

## Features
✅ **Basic game simulation**  
  - ✅ Handles full Coup game flow  
  - ⏳ Optimized to use better game design architecture instead of many branches  
🚧 **Card counting mechanism for pruning search**
  - ✅ Create brute force tracker for validation
  - 🚧 Create memoized card counter for quicker querying of impossible states  
  - ⏳ Front-end visualization  
⏳ **State Probability Tracker**  
  - ⏳ Use a GPU Compute Shader to compute probability of a particular card permutation state  
⏳ **Counterfactual Regret Minimization (CFR)**
  - ⏳ Basic CFR
  - ⏳ CVFPR
  - ⏳ CVFPR + RL  

## Technical Overview
This bot leverages **Counterfactual Regret Minimization (CFR)** to iteratively improve its decision-making process. The goal is to approximate **optimal strategies** by adjusting based on simulated playthroughs.  

Accelerating Nash Equilibrium Convergence in Monte Carlo Settings Through Counterfactual Value Based Fictitious Play  
https://arxiv.org/abs/2309.03084
Student of Games: A unified learning algorithm for both perfect and imperfect information games  
https://arxiv.org/abs/2112.03178
Combining Deep Reinforcement Learning and Search for Imperfect-Information Games  
https://arxiv.org/abs/2007.13544
