# SpedX Spot Margin
This is a Work-In-Progress protocol offering margining services on top of a fully on-chain and crankless spot orderbook called Phoenix. The protocol composes with Phoenix and allows users to access margin and place trades that are settled on Phoenix's orderbooks, i.e on-chain.

Currently, the types of Margin offered is limited to Isolated Margin. This decision has been taken keeping in mind the customizability to create tail markets, with increased liquidation risk of positions. Isolated margin(WIP) coupled with a robust risk and liquidation engine(WIP) allows for dynamic margin requirements, leverage caps, borrow caps, as well as asset and liability weights(adjusted by a discounting factor). So, theoretically, any tail market can be created and trading can be allowed on that market, without the risk of over-leveraging of whales leading to contagions caused by liquidations.

Please keep in mind that this codebase is changing rapidly, and we suggest you not to assume it as the final version. The current codebase is just a fraction of the final codebase. Hence, it is not deployed to mainnet-beta, and is currently on devnet. Once it is completed and audited, it will be deployed to mainnet-beta.

The codebase will be open-source for anyone to view, and we will be accepting contributions to the code soon.