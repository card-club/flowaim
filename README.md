# Flowaim - Opinionated SxT CLI setup tool

When trying to scaffold/setup your cryptographic verified analytics pipeline with SxT this tool is probably for you.

Currently only has 3 commands:

- Setup
- Load
- Destroy 

## Setup

Ask your organisation name and sets up different tables based on how many environment you want to have. Options are dev/test/stag/prod.
Will print out the biscuits per table, but want to change this to a .flowaim.toml config file in the future 

## Load

Initial load of the dummy data in your table for testing. 

## Destroy

Destroy the schema after input. 

