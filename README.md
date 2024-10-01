# Trustful Stellar

Trustful is an innovative and flexible reputation aggregation system designed for the Stellar blockchain. It allows communities and projects to issue digital badges to recognize and reward user contributions.

## Overview

The Trustful Stellar project consists of two main parts:

1. Scorer Factory
2. Scorer

Together, these components form an ecosystem that enables the creation, management, and utilization of customized scoring systems on the Stellar network.

## Architecture

![ScorerContract + Scorer Factory (1)](https://github.com/user-attachments/assets/4e39f4ba-fb09-4ead-a7d3-a3141fab6061)


## 1. Scorer Factory

The Scorer Factory is a contract that facilitates the creation and management of multiple Scorer contracts. Its main functions are:

- Allow the creation of new Scorers in a standardized way by anyway
- Maintain a record of all created Scorers

### Main Functionalities:

- `createScorer`: Creates a new Scorer contract with specific parameters
- `getScorers`: Returns the list of all created Scorers

## 2. Scorer

The Scorer is an individual smart contract that represents a specific scoring system for a community or project.

### Main characteristics of the Scorer:

- Issuance of digital badges as NFTs
- Customizable comparison rules
- Calculation and storage of reputation scores
- Permission management (creator and managers)

### Structures:

- `badges`: ScorerBadge[] - List of badges available in the Scorer
- `userScores`: mapping(address => uint256) - Mapping of user scores

### Main Functionalities:

- `calculateScore`: Calculates a user's score
- `getUserScore`: Returns a user's current score
- `addManager`: Adds a new manager to the Scorer

## General Features of Trustful Stellar

- Customizable comparison rules to adapt to the needs of each community
- Digital badges issued to recognize user contributions
- Reputation scores generated transparently and verifiably on the blockchain
- Flexible system adaptable to various communities and projects on the Stellar network

## How to Use

1. Deploy the Scorer Factory contract on the desired Stellar network.
2. Use the Scorer Factory to create new Scorers as needed.
3. Interact with individual Scorer contracts to manage scores.
4. Use the Scorer Factory to obtain the list of all created Scorers, if necessary.

## Contributing

Contributions are welcome! Please open an issue or pull request for suggestions for improvements or corrections.

## License

[MIT License](LICENSE)
