# Affiliate Program

This program is used to monitor partner fee on-chain. All Mercurial partners have to route user’s requests though this program to track shared fee

## Participate in the affiliate program:

Each partner, who wants to join in the affiliate program, must send the system wallet address to Mercurial foundation. Protocol fee will be distributed based on a negotiated ratio, and sent to the associated token account of above wallet address. 

Note: Admin will send a transaction to create a partner PDA corresponding with partner wallet address and the vault (USDC/USDT, etc)

```
    pub fn init_partner(ctx: Context<InitPartner>, fee_ratio: u64)
```

The accrued fee of the partner will be monitored in partner PDA, and Mercurial admin can also track and send fee to partner. All fee are sent as native token, example: if patner integrates with USDC vault, then admin will send fee as USDC to partner. 

## For new user

If a user is new with this partner, before sending deposit/withdraw transactions, the partner has to send a transaction to init user PDA. 
```
pub fn init_user(ctx: Context<InitUser>)
```

Refer to <a href="https://github.com/mercurial-finance/vault-periphery/blob/affiliate_readme/affiliate/rust-client/src/partner.rs#L8">init_user_instruction</a>

Every time a user deposit/withdraw, the partner has to send along with user PDA and partner PDA to track the yield that user has earned. Then program would know the performance fee per this user and update fee for the partner 

If a user has been routed through the parner, the partner can skip this step. 


## User deposit/withdraw/withdraw_from_strategy

When a user deposits, lp token will be minted to user PDA token account, which only this user can sign the user PDA token account to withdraw the fund. LP tokens are kept in user PDA allowing the program to track the parner fee when the user withdraws. 

Refer to <a href="https://github.com/mercurial-finance/vault-periphery/blob/affiliate_readme/affiliate/rust-client/src/user.rs">sample instructions</a>
