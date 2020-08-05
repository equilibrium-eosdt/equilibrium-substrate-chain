
![ ](logo.png)

## Equilibrium
   Equilibrium is introducing the first cross-chain money market that combines pooled lending with synthetic asset generation and trading.  

   Currently, 90% of the Defi market is locked on Ethereum. However, Ethereum only comprises 12% of crypto overall. Vast market potential for DeFi remains untapped. Equilibrium will unlock the remaining full potential of DeFi. 


   Using Polkadot’s advanced substrate technology to create its own blockchain, Equilibrium will extend its DeFi product line to a one-stop cross-chain money market. It is designed to end market segmentation and will let users conveniently fill all their DeFi needs in one place -- uniting decentralized liquidity pools, enabling cross-chain lending with built-in synthetic assets, and advanced price discovery and bailout mechanics to ensure maximum liquidity. 

   Users will be able to trade multiple crypto assets from Equilibrium’s native decentralized exchange and use its convenient interface to manage their assets across multiple blockchain protocols, without new logins. 

   Please read Equilibrium’s [White Paper](https://equilibrium.io/docs/Equilibrium_WP_101.pdf) to learn more and visit the Equilibrium website: [www.equilibrium.io](https://www.equilibrium.io)

## Our roadmap

| Stage            |                                                                              Features                                                                              |                                                                        User can                                                                        |
| ---------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------: | :----------------------------------------------------------------------------------------------------------------------------------------------------: |
| **Stage&#160;1** |                                             Launch of the Equilibrium’s substrate </br> Interconnection with Ethereum                                              |                                      Utilize ETH & DOT assets </br> Earn on bailing out </br>  Borrow stablecoins                                      |
| **Stage&#160;2** | Equilibrium’s substrate becomes Polkadot parachain </br> Launch of pooled lending </br> Interconnection with Bitcoin and COSMOS </br> MVP of the interoperable DEX |                            Utilize additionally BTC & COSMOS-based assets </br> Lend and borrow crypto </br>  Trade on DEX                             |
| **Stage&#160;3** |                Interconnection with EOS and Tezos </br> Synthetic asset generation </br> Beta version of the interoperable DEX </br> Margin trading                | Utilize additionally EOS & XTZ assets </br> Borrow synthetic assets pegged to multiple fiat currencies, commodities, stocks etc. </br> Trade on margin |
| **Stage&#160;4** |                                                        More blockchains interconnected  </br> Delta hedging                                                        |                                                Utilize more assets  </br> Hedge portfolio automatically                                                |


## Run

### Single node development chain

Purge any existing developer chain state:

```bash
curl https://getsubstrate.io -sSf | bash -s -- --fast

./scripts/init.sh
```

```bash
cargo build --release
```

```bash
cargo check
```

cargo  tests

```bash
cargo check --tests
```

Launch all tests

```bash
cargo t
```

```bash
./target/release/eq-node purge-chain --dev
```

Start a development chain with:

```bash
./target/release/eq-node --dev
```

https://polkadot.js.org/apps/#/explorer

Select network:
Local Node (Own, 127.0.0.1:9944)

Change settings for custom types:
Settings -> Developer ->

```json
{
   "Keys":"SessionKeys2",
   "Balance":"u64",
   "FixedI64":"i64",
   "SignedBalance":{
      "_enum":{
         "Positive":"Balance",
         "Negative":"Balance"
      }
   },
   "Currency":{
      "_enum":[
         "Unknown",
         "Usd",
         "Eq",
         "Eth",
         "Btc",
         "Eos"
      ]
   },
   "BalancesAggregate":{
      "total_issuance":"Balance",
      "total_debt":"Balance"
   }
}
```

## License

Equilibriun substrate is [GPL 3.0 licensed].