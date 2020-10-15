# compare-csv

A CLI tool for comparing CSV files as sets of records.

## Examples

### Example 1

You can select columns for comparing records.
The program sees first and fifth columns in this example.

```
% cargo run -- --source-file examples/base.csv \
--source-columns "date:Date,description,account,unit,price:Decimal" \
--target-file examples/delta.csv \
--target-columns "date:Date,description,account,unit,price:Decimal" \
--predicate "(date, price) = (date, price)"
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/csvdiff --source-file examples/base.csv --source-columns 'date:Date,description,account,unit,price:Decimal' --target-file examples/delta.csv --target-columns 'date:Date,description,account,unit,price:Decimal' --predicate '(date, price) = (date, price)'`
# 2020-09-12,-1000
< 2020-09-12,Lunch,Liabilities:CreditCard,JPY,-1000
# 2020-09-20,-4500
> 2020-09-20,Adobe,Liabilities:CreditCard,JPY,-4500
```

### Example 2

You can convert a column value negative when the column type is `Decimal`.

```
% cargo run -- --source-file examples/postings.csv \
--source-columns "date:Date,description,account,unit,price:Decimal" \
--target-file examples/history.csv \
--target-columns "date:Date,description,price:Decimal" \ 
--predicate "(date, -price) = (date, price)"
    Finished dev [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/csvdiff --source-file examples/postings.csv --source-columns 'date:Date,description,account,unit,price:Decimal' --target-file examples/history.csv --target-columns 'date:Date,description,price:Decimal' --predicate '(date, -price) = (date, price)'`
# 2020-09-11,100
< 2020-09-11,Vending Machine,Liabilities:CreditCard,JPY,-100
< 2020-09-11,Vending Machine,Liabilities:CreditCard,JPY,-100
> 2020-09-11,Mobile,100
# 2020-09-16,1000
> 2020-09-16,Mobile,1000
# 2020-09-18,2000
> 2020-09-18,Mobile,2000
```

## To-Do

- [ ] Addition, subtraction, multiplication and division in predicate.
- [ ] Well-written README.
