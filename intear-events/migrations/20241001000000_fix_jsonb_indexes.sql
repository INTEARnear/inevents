DROP INDEX trade_swap_testnet_idx_balance_change_token;
CREATE INDEX trade_swap_testnet_idx_balance_change_token ON trade_swap_testnet USING GIN(balance_changes jsonb_ops);
DROP INDEX trade_swap_idx_balance_change_token;
CREATE INDEX trade_swap_idx_balance_change_token ON trade_swap USING GIN(balance_changes jsonb_ops);
