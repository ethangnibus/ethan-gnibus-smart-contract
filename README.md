# Ethan Gnibus Smart Contract
## What is a Smart Contract?
To keep it simple, [Smart contracts](https://en.wikipedia.org/wiki/Smart_contract) control the logic of operations that happen on [blockchains](https://en.wikipedia.org/wiki/Blockchain). Smart contracts track their internal history through states that are saved when actions are preformed. A smart contract that tracks N states could be visualized below:

![image](https://user-images.githubusercontent.com/59241452/147776266-d2e5fb94-5cbe-4da8-a9e3-71c1167dc358.png | height=100)

## Why the Ethan Gnibus Smart Contract is special? 
The Ethan Gnibus Smart Contract is built on [Terra](https://docs.terra.money/), the leading decentralized and open-source public blockchain protocol for [algorithmic stablecoins](https://en.wikipedia.org/wiki/Stablecoin).
### Each state in this particular contract will contain addresses with corresponding scores.
We could represent this as the following, where 1 <= K <= N:

![image](https://user-images.githubusercontent.com/59241452/147776364-7ffa0f01-0180-4277-bc3e-80e272941913.png  | height=100)

As we can see, the architecture of the state looks very similar to that of a Dictionary in Python.

### Each state will also store an identifier of the user who created the instance of the smart contract.
So if Alice created one instance of the smart contract we would have:

![image](https://user-images.githubusercontent.com/59241452/147776420-0828f25c-c159-48d7-a8ff-69ae858db588.png)

And if Bob created another instance we would have:

![image](https://user-images.githubusercontent.com/59241452/147776457-fe829ac6-afff-45ce-9aae-3cf9fd08f23c.png)

### Queries
Another feature of the Ethan Gnibus Smart Contract is that it implements queries that can extract information from its states without editing the state's contents.
#### Query 1: Getting the owner of an instance of a smart contract
If there exists a state history like the following, it is useful to find out who initialized the smart contract.

![image](https://user-images.githubusercontent.com/59241452/147776486-a7a96b3d-052d-4c63-b9ac-ac14ce63f030.png)

To do this, this contract implements a query that returns the owner field of the current state.

![image](https://user-images.githubusercontent.com/59241452/147776848-c5843c56-3bec-45d4-af4b-350aeecd145d.png)

#### Query 2: Getting a score using a corresponding address
Over time the addresses in the state history will populate so extracting information from the corresponding scores could be useful. This contract uses the current state to extract corresponding scores.

![image](https://user-images.githubusercontent.com/59241452/147776560-063f5bc8-dbc0-490a-bec9-2ef1623f7909.png)

### Executes
Another feature of the Ethan Gnibus Smart Contract is that it implements executable commands that can update it's internal state.
#### Execute 1: The owner of the contract can edit the score of an address
The owners of contract instances could edit the scores at corresponding addresses. For example, if the state history is:

![image](https://user-images.githubusercontent.com/59241452/147776583-0e35eade-c0d5-4004-8d50-50646f4ea284.png)

The owner could update the score at Address_1 to be 99. After this the state history will become:

![image](https://user-images.githubusercontent.com/59241452/147776623-0032270c-daf2-4ae2-a44c-20ef5f9841a5.png)

#### Execute 2: Anyone can update the state with a new address
Any user could add a score at a new address. For example, if the state history is:

![image](https://user-images.githubusercontent.com/59241452/147777001-18bb4f9f-e685-4512-be9a-26c943e96028.png)

Any user could make Address_3 and store 10 in it:

![image](https://user-images.githubusercontent.com/59241452/147777029-001fea5b-4aad-46ee-a90c-6fbe5e1907f8.png)

## Footer

This project is based on the [CosmWasm Starter Pack](https://github.com/InterWasm/cw-template).
