# Relayer Game
[![Build Status](https://travis-ci.com/yanganto/s3handler.svg?branch=master)](https://travis-ci.com/yanganto/relayer-game)

`relayer-game` is a tool to simulate the optimized game for relayers in Darwinia Network. 
In order to ban the replay who lies, and also we want to help make thing finalize as soon as possible, 
this tool can easily to change three important equations and load from different scenario to simulate the relayer game.  

## Scenario
In this tool we assume the target chain is Ethereum, however you can simulate different chain by changing parameters.
All the behavior of relayers, and the parameters are described a in a yaml file. 
You can easily load the scenario file to simulate the result. 
There are some example scenario files listed in [scenario](./scenario).

There are two different gaming models, one is `relayers-only` mode, and the other is `relayer-challenger` mode.
In relayers-only mode, when someone is not accepted the block submitted by other relayer, he should submit the correct block to express his opinion.
In relayer-challenger mode, when someone is not accepted the block submitted by other relayer, he just put a challenge on chain to express his opinion.

If there is any `[[challengers]]` in scenario file, the scenario will run in relayer-challenger mode.
The `scenario/challenge.yml` is a scenario for one relayer and one challenger, you may run it with `-v` option to know more about this.


### relayers-only mode
There are 3 rules in relayers-only mode.
1. Anyone can relay a ethereum header on the darwinia chain
2. Confirmed or open a round of game
    - if there is only one block (or same blocks) over the challenge time, this block is confirmed.  (2-1)
    - else (there are different blocks in the same block height) the game is starts
      - everyone in the game should submit the block based on the `target function` until closed (2-2-1)
        - Once the a block according target function submit the next round of gamae is started, and become recursive in 2
    - anyone can get in the game in any round but need to participate until the game closed
3. Game closed
    - In following two condition, that all of the submission in the game from a relayer are ignored. (3-1)
      - if someone in the game did not keep submit the blocks in following rounds in the game
      - if the block of someone in the game is verified fail by hash
    - Because some submission are ignored, if all blocks in each round become the only or the same, this block is confirmed, and game is closed. (3-2)

Here is some visualized example with two relayer `Evil` and `Honest`, `Evil` is a bad guy some times relay incorrect header, `Honest` is always relay a correct header.

This is a plot to show Ethereum, G is genesis block or a block already confirmed on Darwinia chain before the game start.
```
         G==========================================>
```

Here is the first submit, `Evil` relay a block with lie(`L`), and `Honest` find out this and relay a truth block with the same ehereum block height,  
so the game starts.  In the meanwhile, the chain can not determine which block is truth or lied, all the information on chain is that there are two different blocks with same block height, at least one of the block with incorrect information.
```
         G======================================1===>
Evil                                            L
Honest                                          H
```
Based on `target function`, the `Evil` and `Honest` should submit the header on the `2` position (adopted rule 2-2-1.).
```
         G==================2===================1===>
Evil                                            L
Honest                                          H
```
#### From here, the game will become 3 kinds of scenarios, 
  - `Evil` has no response on 2,
  - `Evil` submit a true block on 2 honestly.
  - `Evil` submit a block still with lie on 2.

##### In the first scenario (`Evil` has no response on 2),
the `Honest` will submit a header on 2
```
         G==================2===================1===>
Evil                                            L
Honest                      H                   H
```
And waiting the challenge time over, the lie block from `Evil` will be removed (adopted rule 3-1), 
and the only block in each round will be confirmed (denote with 'C') (adopted rule 3-2).
```
         G==================2===================1===>
Evil                                            -
Honest                      C                   C
```

##### In the second scenario (`Evil` submit a block on 2 honestly),
the `Honest` will submit a header on 2
```
         G==================2===================1===>
Evil                        H                   L
Honest                      H                   H
```

And waiting the challenge time over, 
the blocks (the same) in submit round 2 are all confirmed. (adopted rule 2-1)
And `Evil` and `Honest` are still in the game and base on `target_function`, 
that should submit header on position 3(adopted rule 2-2-1.).
```
         G==================2=========3=========1===>
Evil                        C                   L
Honest                      C                   H
```

##### In the third scenario (`Evil` submit a block still with lie on 2),
the `Honest` will submit a correct header on 2.
```
         G==================2===================1===>
Evil                        L                   L
Honest                      H                   H
```
And there is nothing confirmed without different opinions, 
so base on the `target_function` the position 3 should be submit by `Evil` and `Honest`.
```
         G=======3==========2===================1===>
Evil                        L                   L
Honest                      H                   H
```
`Evil` and `Honest` can start to submit the block on position 3 when they have different opinion on position 2, 
but the challenge time of submit round 3 will be star counting after run out the challenge time of submit round 2.


#### Conclusion of relayer-only mode
- In the first scenario, the game is closed.  
- In the second and third scenario, the game is still going and will be convergence some where between `G` and `1`.

Once the `Evil` goes into contradictory. All of the bond from `Evil` will be slashed, and the slash to reward can be distributed with different functions.
In the model, no mater there are how many malicious relayers, one honest relayer always response correct information will win the game.

### relayer-challenger mode
In relayer-challenger mode, when someone is not accepted the block submitted by other relayer, he just put a challenge on chain to express his opinion.
And currently, we are not consider the challenger do evil scenario.
However, there is still a bond for the challenger to challenge.
1. Any relayer can relay a ethereum header on the darwinia chain
2. Any challenger can challenge the relayer and open the game
    - challenger needs to bond some value for each challenge before the half challenge time over
    - relayer needs to submit the next specified block by `target function`
3. Game closed
    1. The challenger stop challenging, the relayer wins and challenger will be slashed
    2. The relayer can not provided the target block block pass the validation before the next half challenge time over

Here is some visualized example with relayer `Evil` and challenger `Challenger`, `Evil` is a bad guy some times relay incorrect header, `Challenger` is honest challenger challenge with the correct opinion.

This is a plot to show Ethereum, G is genesis block or a block already confirmed on Darwinia chain before the game start.
```
             G==========================================>
```

Here is the first submit, `Evil` relay a block with lie(`L`), and `Challenger` find out this block is not correct and challenge it with `0`, so the game starts.  
In the meanwhile, the chain still can not determine which block is truth or lied, all the information on chain is that there is a block with dispute.
```
             G======================================1===>
Evil                                                L
Challenger                                          0
```

Based on `target function`, the `Evil` should submit the block on position 2.
```
             G=================2====================1===>
Evil                                                L
Challenger                                          0
```
#### From here, the game will become 3 kinds of scenarios, 
  - `Evil` has no response on 2,
  - `Evil` submit a block on 2 honestly.
  - `Evil` submit a block still with lie on 2.

##### In the first scenario (`Evil` has no response on 2),
If `Evil` is not response before the half challenge time over, 
the `Challenger` will win the game and the bond of `Evil` in position 1 will become the be slashed and become reward for `Challenger`.

##### In the second scenario (`Evil` submit a block on 2 honestly),
If `Evil` submit a correct block in position 2, the challenger will challenge with `0` on position 1 and '1' on position 2.
```
             G=================2====================1===>
Evil                           H                    L
Challenger                     1                    0
```
Such that, based on `target function`, the next target will be between the position 1 and position 2.
```
             G=================2==========3=========1===>
Evil                           H                    L
Challenger                     1                    0
```
##### In the third scenario (`Evil` submit a block still with lie on 2),
If `Evil` submit a correct block in position 2, the challenger will challenge with `0` on position 1 and '0' on position 2.
```
             G=================2====================1===>
Evil                           L                    L
Challenger                     0                    0
```
Such that, based on `target function`, the next target will be between the genesis and position 2.
```
             G=======3=========2====================1===>
Evil                           L                    L
Challenger                     0                    0
```
#### Conclusion of relayer-challenger mode
- In the first scenario, the game is closed.  
- In the second and third scenario, the game is still going and will be convergence some where between `G` and `1`.
  - Following are the assumption, that challenger will beat the evil relayer
    - The fake blocks are not easy to pass validation blocks when near by
    - challenger is not collusion with the evil relayer

Once the `Evil` goes into contradictory. All of the bond from `Evil` will be slashed, and the game is closed.
Please note there is no correct block on position 1 after the game closed, so there may be multiple parallel relayer-challenger games on chain to keep the bridge works.

## General parameters
- `title ` (optional)
  - The title for this scenario will print on the console
- `F` (optional)
  - The block producing factor for Darwinia / Ethereum
  - For example: 2.0, that means that Darwinia produce 2 blocks and Ethereum produce 1 block.

## Specify functions type
- `challenge_function`
  - Once a relayer submit a header and challenge the time in blocks after the calculated value from challenge function, 
    Darwinia network will deem this header is valided and become a best header.  
  - Current support: integer number, linear   
  - For example: '10', that means a submit block will be deem to relayed and finalized after 10 Darwinia blocks.
  - For example: `challenge_linear`, that means a submit block will wait according the linear function, and the parameters of function need to provide.

- `target_function`
  - Once there is dispute on any header, the relayer should submit the next Ethereum block target as calculated.  
  - Current support: half
  - For example: 'half', that means the next target will be `(submited_ethereum_block_height - relayed_ethereum_block_height) / 2`

- `bond_function`
  - The bond function will increase the bond to improve speed of the finality, and the cost of keeping lie will be enormous.  
  - Current support: float number, linear   
  - For example: '10.0', that means the bond of each submit is always 10.0.

- `reward_function`
  - The reward from relayers including the treasury and the slash from the lie relayers (attackers)
  - It may be reasonable when that the slash part of reward should be the same as bond functions, 
    but both the first round submit and the second round submit are to help to beat the attackers in the first round, 
    so the slash from the first round sumit may split some portion for the honest relayers in the second round.
  - And also it may be that the treasury part is only for the last submit rounds, if the slash never split to the next round
    - the treasury part is from the fee of redeem action, but it will be a debt without limitation in simulation

## Initialize status of Darwinia and Ethereum
suffix `d`: block difference between last block number relayed on Darwinia, suffix `e`: block difference between last related block number of Ethereum
- `Dd`(optional)
  - the block difference between last block number relayed on Darwinia, 
- `De`(optional)
  - the block difference between last related block number of Ethereum

## Relayers' Chose
Relays will not always be honest, s/he may some times cheat or not response.  
The following parameters are used for relayers
- `[[relayers]]` 
  - `name` (optional)
    - if name is not provided, the relayer will be name with serial numbers
  - `choice`
    - relayer may be response as `H`(Honest), `L`(Lie), `N`(No response)
    - if the length of chose are shorter than other relayers, it will be deem to no response.  

We assume there always is a good guy to relay the correct headers, and the guy will name `Darwinia`, 
and this relayer will be automatic add into the scenario when load from configure file, 
so please avoid to use this name for the relayer.

## Challengers' Chose
Challengers will always be honest in current model
The following parameters are used for challenger
- `[[challengers]]` 
  - `name` (optional)
    - if name is not provided, the relayer will be name with serial numbers
  - `choice`
    - challenger may be response as following
      - `1`(agree with relayer, this means relayer is honest at this round)
      - `0`(disagree with relayer, this means relayer lies at this round)

## Parameters of Equation
The three function can use different equations, base on the function setting, following parameters of function should be filled.
- `[challenge_linear]`
  - the linear equation for challenge function
  - `challengeing block = int(min(Wd * D, Md) + min(We * E, Me)) + C`
  - suffix `d`: block difference between last block number relayed on Darwinia, suffix `e`: block difference between last related block number of Ethereum
  - D is the distance in Darwinia chain
  - E is the distance in Ethereum chain
  - W is the weight for that portion
  - M is the maximum value for that portion

- `[bond_linear]`
  - the linear equation for bond function
  - `bond = min(W * E, M) + C`
  - W is the weight 
  - M is the maximum value for the variable part

- `[reward_split]`
  - Split the slash for reward the honest relayers in two rounds
  - slash value of submit round will take P as reward in current round, and leave (1-P) for the next round
  - P is the portion 
  - the slash value for this submit round is slash value * P
  - the slash value for this next submit round is slash value * (1 - P)

- `[reward_treasury_last]`
  - The slash for reward the honest relayers in the same round
  - The treasury will reward the relayers in the last submit round, because there is no attacker(lie relayers) in the last round.
  - C is the constant of the reward from treasury

## Build & Run
This executable is written in Rust, it can be easily to build and run with cargo command.  
```
cargo build --release
```
then the binary will be placed in ./target/release, you can run this command with scenario file as following command.  
```
./target/release/relayer-game scenario/basic.yml
```
Also, you can put `-v` option to see all status in each round of submit.
```
./target/release/relayer-game -v scenario/basic.yml
```
following picture is the example what you will see with verbose flag
![snapshot](https://raw.githubusercontent.com/yanganto/relayer-game/master/demo-vebose.png)

Besides, you can patch some equation parameters with option `p`, for examples.
```
./target/release/relayer-game -p challenge_linear.C=9 challenge_linear.Wd=10.0 -- scenario/basic.yml
```
Currently, all parameters in `challenge_linear` and `bond_linear`, and also the values of `challenge_function` and `bond_function` can be patched.  
The challenge times(in blocks), and the bonds for each round will show as plot help you to modify the equation.  
![snapshot](https://raw.githubusercontent.com/yanganto/relayer-game/master/demo.png)

After running this tool, the reward and slash from each relayer will show as following picture.
![snapshot](https://raw.githubusercontent.com/yanganto/relayer-game/master/demo2.png)

If you want to use this tool without plot with a smaller binary, please use `--no-default-features` option when building.
```
cargo build --release --no-default-features
```

## Develop and Document
This project has document, you can use this command to show the document on browser.
`cargo doc --no-deps --open`
If you want to add more equation for different function, you can take a look the trait in [bond](./src/bond/mod.rs), [challenge](./src/challenge/mod.rs), [target](./src/target/mod.rs).
The `Equation` trait and `ConfigValidate` will guild you to add you customized equation. 
