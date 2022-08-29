
# CLI for mercurial vault

## Build

`cargo build`

## Command

Check command with `../target/debug/rust-client --help`

```
USAGE:
    rust-client [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help                                Print help information
        --provider.base <BASE>                
        --provider.cluster <CLUSTER>          Cluster override
        --provider.program_id <PROGRAM_ID>    Program id override
        --provider.token_mint <TOKEN_MINT>    Token mint override
        --provider.wallet <WALLET>            Wallet override

SUBCOMMANDS:
    deposit                   
    fund-partner              
    get-unlocked-amount       
    help                      Print this message or the help of the given subcommand(s)
    init-partner              
    init-user                 
    show                      
    view-partner              
    withdraw                  
    withdraw-from-strategy               
```


## Example

```
cargo run -- show --provider.token_mint So11111111111111111111111111111111111111112


cargo run -- init-partner 5unTfT2kssBuNvHPY6LbJfJpLqEcdMxGYLWHwShaeTLi --provider.token_mint 9NGDi2tZtNmCCp8SVLKNuGjuWAVwNF3Vap5tT8km5er9 

cargo run -- init-partner 7236FoaWTXJyzbfFPZcrzg3tBpPhGiTgXsGWvjwrYfiF --provider.token_mint So11111111111111111111111111111111111111112 

cargo run -- init-partner EKs1F8DTYA9pREXExCSsmCG4Z16DtbG99QHgfNPDLq4J --provider.token_mint So11111111111111111111111111111111111111112 

cargo run -- init-partner-all-vault H43eoSDLE2A5QNbfxesdtg9P9MqfPEFh2WD2rSVFHaHd --provider.token_mint So11111111111111111111111111111111111111112 


../target/debug/rust-client update-fee-ratio 9pxiKDu6yFGxSxKiFERV31UpA6y4BpbuGeBjAKeLic8B 2000 --provider.token_mint So11111111111111111111111111111111111111112 

cargo run -- init-user 7236FoaWTXJyzbfFPZcrzg3tBpPhGiTgXsGWvjwrYfiF --provider.token_mint So11111111111111111111111111111111111111112 

cargo run -- deposit 100000000 7236FoaWTXJyzbfFPZcrzg3tBpPhGiTgXsGWvjwrYfiF --provider.token_mint So11111111111111111111111111111111111111112

../target/debug/rust-client withdraw 100 9pxiKDu6yFGxSxKiFERV31UpA6y4BpbuGeBjAKeLic8B --provider.token_mint So11111111111111111111111111111111111111112

../target/debug/rust-client withdraw-from-strategy 100 8fSuEU6mnggaSZsYQsSbUS1ytLFPGbFAvANKYd4QWAtx 9pxiKDu6yFGxSxKiFERV31UpA6y4BpbuGeBjAKeLic8B --provider.token_mint So11111111111111111111111111111111111111112

../target/debug/rust-client fund-partner 9pxiKDu6yFGxSxKiFERV31UpA6y4BpbuGeBjAKeLic8B 37 --provider.token_mint So11111111111111111111111111111111111111112

cargo run -- view-partner 9pxiKDu6yFGxSxKiFERV31UpA6y4BpbuGeBjAKeLic8B --provider.token_mint So11111111111111111111111111111111111111112

../target/debug/rust-client view-user 9pxiKDu6yFGxSxKiFERV31UpA6y4BpbuGeBjAKeLic8B --provider.token_mint So11111111111111111111111111111111111111112
```