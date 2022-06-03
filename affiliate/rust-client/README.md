
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
../target/debug/rust-client show --provider.token_mint So11111111111111111111111111111111111111112

../target/debug/rust-client init-partner 4sBMz7zmDWPzdEnECJW3NA9mEcNwkjYtVnL2KySaWYAf 2000 --provider.token_mint So11111111111111111111111111111111111111112 

../target/debug/rust-client init-user 4sBMz7zmDWPzdEnECJW3NA9mEcNwkjYtVnL2KySaWYAf --provider.token_mint So11111111111111111111111111111111111111112 

../target/debug/rust-client deposit 100 4sBMz7zmDWPzdEnECJW3NA9mEcNwkjYtVnL2KySaWYAf --provider.token_mint So11111111111111111111111111111111111111112

../target/debug/rust-client withdraw 100 4sBMz7zmDWPzdEnECJW3NA9mEcNwkjYtVnL2KySaWYAf --provider.token_mint So11111111111111111111111111111111111111112

../target/debug/rust-client withdraw-from-strategy 100 8fSuEU6mnggaSZsYQsSbUS1ytLFPGbFAvANKYd4QWAtx 4sBMz7zmDWPzdEnECJW3NA9mEcNwkjYtVnL2KySaWYAf --provider.token_mint So11111111111111111111111111111111111111112

../target/debug/rust-client fund-partner 4sBMz7zmDWPzdEnECJW3NA9mEcNwkjYtVnL2KySaWYAf 37 --provider.token_mint So11111111111111111111111111111111111111112

../target/debug/rust-client view-partner 4sBMz7zmDWPzdEnECJW3NA9mEcNwkjYtVnL2KySaWYAf --provider.token_mint So11111111111111111111111111111111111111112

../target/debug/rust-client view-user 4sBMz7zmDWPzdEnECJW3NA9mEcNwkjYtVnL2KySaWYAf --provider.token_mint So11111111111111111111111111111111111111112
```