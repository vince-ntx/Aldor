# Bank API
## Goal
Build a bank to receives deposits and issue loans.    

## Features
x open, close an Account
    x fetch accounts
x send funds between users
x make deposits/withdrawals
x earn a profit for the bank
- get a loan

## Objects
### Accounts
- Account - type of account (checking, savings, etc.)
- AccountHolder - the person
- Bank entity
### Transactions
- deposits
- withdrawals
### Loan - Amortized
- Principal: BigDecimal
- Term: how long the loan lasts
    - Maturity Date - Issue Date
- Annual Interest Rate: 
    - Annual interest rate for this loan. Interest is calculated each period on the current outstanding balance of your loan. The periodic rate is your annual rate divided by the number of periods per year.
- Payment Frequency
    - Monthly
    - Quarterly
- Compound Frequency
    
Calculator: https://www.mybct.com/calculator/complex-loan
    

### Earnings for bank and customer


