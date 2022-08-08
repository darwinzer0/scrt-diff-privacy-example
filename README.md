# Differential privacy example on Secret Network

This repository contains an example of differentially private queries on encrypted data stored on Secret Network. It demonstrates COUNT and MEAN queries which are fuzzied using the Laplace mechanism to add noise to the result.

## Running test

Follow the instructions to build the contract in the `contract` folder. Start the dev chain (cosmwasm v1.0 version) and run the test script in `test`. More instructions for building and running tests, respectively, are in the README.md files of each subdirectory.

## Features

The test script uploads a set of observations to the contract (sample data from the [Iris dataset](https://en.wikipedia.org/wiki/Iris_flower_data_set)) and calculates fuzzy count and average values for the observations.

The [substrate-fixed](https://github.com/encointer/substrate-fixed) crate was used to implement noise calculation using fixed-point arithmetic. (The Laplace mechanism requires logarithm on a unit interval random number).

Contract initialization sets the epsilon value (higher epsilon gives closer to true value / less privacy, lower epsilon adds more noise / more privacy), and a privacy budget. Each query will use some of the privacy budget, and once it is exhausted no more queries are allowed on the data. 

The queries are implemented as `exec` functions in the contract, because they need to update how much of the privacy budget is left.