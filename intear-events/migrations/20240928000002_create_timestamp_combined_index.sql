CREATE INDEX nft_transfer_idx_timestamp_old_owner_id ON nft_transfer(timestamp, old_owner_id);
CREATE INDEX nft_transfer_idx_timestamp_new_owner_id ON nft_transfer(timestamp, new_owner_id);
CREATE INDEX nft_transfer_idx_timestamp_contract_id ON nft_transfer(timestamp, contract_id);
CREATE INDEX nft_mint_idx_timestamp_owner_id ON nft_mint(timestamp, owner_id);
CREATE INDEX nft_mint_idx_timestamp_contract_id ON nft_mint(timestamp, contract_id);
CREATE INDEX nft_burn_idx_timestamp_owner_id ON nft_burn(timestamp, owner_id);
CREATE INDEX nft_burn_idx_timestamp_contract_id ON nft_burn(timestamp, contract_id);
CREATE INDEX potlock_donation_idx_timestamp_donor_id ON potlock_donation(timestamp, donor_id);
CREATE INDEX potlock_donation_idx_timestamp_project_id ON potlock_donation(timestamp, project_id);
CREATE INDEX potlock_donation_idx_timestamp_referrer_id ON potlock_donation(timestamp, referrer_id);
CREATE INDEX potlock_pot_project_donation_idx_timestamp_pot_id ON potlock_pot_project_donation(timestamp, pot_id);
CREATE INDEX potlock_pot_project_donation_idx_timestamp_donor_id ON potlock_pot_project_donation(timestamp, donor_id);
CREATE INDEX potlock_pot_project_donation_idx_timestamp_project_id ON potlock_pot_project_donation(timestamp, project_id);
CREATE INDEX potlock_pot_project_donation_idx_timestamp_referrer_id ON potlock_pot_project_donation(timestamp, referrer_id);
CREATE INDEX potlock_pot_donation_idx_timestamp_pot_id ON potlock_pot_donation(timestamp, pot_id);
CREATE INDEX potlock_pot_donation_idx_timestamp_donor_id ON potlock_pot_donation(timestamp, donor_id);
CREATE INDEX potlock_pot_donation_idx_timestamp_referrer_id ON potlock_pot_donation(timestamp, referrer_id);
CREATE INDEX trade_pool_idx_timestamp_trader ON trade_pool(timestamp, trader);
CREATE INDEX trade_pool_idx_timestamp_pool ON trade_pool(timestamp, pool);
CREATE INDEX trade_pool_idx_timestamp_token_in ON trade_pool(timestamp, token_in);
CREATE INDEX trade_pool_idx_timestamp_token_out ON trade_pool(timestamp, token_out);
CREATE INDEX trade_swap_idx_timestamp_trader ON trade_swap(timestamp, trader);
CREATE INDEX trade_swap_idx_timestamp_balance_change_token ON trade_swap(timestamp, balance_changes);
CREATE INDEX trade_pool_change_idx_timestamp_pool_id ON trade_pool_change(timestamp, pool_id);
CREATE INDEX price_pool_idx_timestamp_pool_id ON price_pool(timestamp, pool_id);
CREATE INDEX price_pool_idx_timestamp_token0 ON price_pool(timestamp, token0);
CREATE INDEX price_pool_idx_timestamp_token1 ON price_pool(timestamp, token1);
CREATE INDEX price_token_idx_timestamp_token ON price_token(timestamp, token);
CREATE INDEX newcontract_nep141_idx_timestamp_account_id ON newcontract_nep141(timestamp, account_id);
CREATE INDEX socialdb_index_idx_timestamp_account_id ON socialdb_index(timestamp, account_id);
CREATE INDEX socialdb_index_idx_timestamp_index_type ON socialdb_index(timestamp, index_type);
CREATE INDEX socialdb_index_idx_timestamp_index_key ON socialdb_index(timestamp, index_key);
CREATE INDEX log_nep297_idx_timestamp_account_id ON log_nep297(timestamp, account_id);
CREATE INDEX log_nep297_idx_timestamp_predecessor_id ON log_nep297(timestamp, predecessor_id);
CREATE INDEX log_nep297_idx_timestamp_event_standard ON log_nep297(timestamp, event_standard);
CREATE INDEX log_nep297_idx_timestamp_event_version ON log_nep297(timestamp, event_version);
CREATE INDEX log_nep297_idx_timestamp_event_event ON log_nep297(timestamp, event_event);
CREATE INDEX log_text_idx_timestamp_account_id ON log_text(timestamp, account_id);
CREATE INDEX log_text_idx_timestamp_predecessor_id ON log_text(timestamp, predecessor_id);
CREATE INDEX new_memecooking_meme_idx_timestamp_meme_id ON new_memecooking_meme(timestamp, meme_id);
CREATE INDEX new_memecooking_meme_idx_timestamp_owner ON new_memecooking_meme(timestamp, owner);
CREATE INDEX new_memecooking_meme_testnet_idx_timestamp_meme_id ON new_memecooking_meme_testnet(timestamp, meme_id);
CREATE INDEX new_memecooking_meme_testnet_idx_timestamp_owner ON new_memecooking_meme_testnet(timestamp, owner);
CREATE INDEX moretps_testnet_idx_timestamp_claimed_account_id ON moretps_testnet(timestamp, claimed_account_id);
CREATE INDEX moretps_testnet_idx_timestamp_claimed_parent_account_id ON moretps_testnet(timestamp, claimed_parent_account_id);
CREATE INDEX moretps_testnet_idx_timestamp_round_account_id ON moretps_testnet(timestamp, round_account_id);
CREATE INDEX moretps_testnet_idx_timestamp_round_parent_account_id ON moretps_testnet(timestamp, round_parent_account_id);
CREATE INDEX moretps_idx_timestamp_claimed_account_id ON moretps(timestamp, claimed_account_id);
CREATE INDEX moretps_idx_timestamp_claimed_parent_account_id ON moretps(timestamp, claimed_parent_account_id);
CREATE INDEX moretps_idx_timestamp_round_account_id ON moretps(timestamp, round_account_id);
CREATE INDEX moretps_idx_timestamp_round_parent_account_id ON moretps(timestamp, round_parent_account_id);
CREATE INDEX block_info_testnet_idx_timestamp_timestamp ON block_info_testnet(timestamp, timestamp);
CREATE INDEX block_info_testnet_idx_timestamp_block_hash ON block_info_testnet(timestamp, block_hash);
CREATE INDEX block_info_testnet_idx_timestamp_block_producer ON block_info_testnet(timestamp, block_producer);
CREATE INDEX block_info_idx_timestamp_timestamp ON block_info(timestamp, timestamp);
CREATE INDEX block_info_idx_timestamp_block_hash ON block_info(timestamp, block_hash);
CREATE INDEX block_info_idx_timestamp_block_producer ON block_info(timestamp, block_producer);
CREATE INDEX memecooking_deposit_idx_timestamp_meme_id ON memecooking_deposit(timestamp, meme_id);
CREATE INDEX memecooking_deposit_idx_timestamp_trader ON memecooking_deposit(timestamp, trader);
CREATE INDEX memecooking_deposit_testnet_idx_timestamp_meme_id ON memecooking_deposit_testnet(timestamp, meme_id);
CREATE INDEX memecooking_deposit_testnet_idx_timestamp_trader ON memecooking_deposit_testnet(timestamp, trader);
CREATE INDEX memecooking_withdraw_idx_timestamp_meme_id ON memecooking_withdraw(timestamp, meme_id);
CREATE INDEX memecooking_withdraw_idx_timestamp_trader ON memecooking_withdraw(timestamp, trader);
CREATE INDEX memecooking_withdraw_testnet_idx_timestamp_meme_id ON memecooking_withdraw_testnet(timestamp, meme_id);
CREATE INDEX memecooking_withdraw_testnet_idx_timestamp_trader ON memecooking_withdraw_testnet(timestamp, trader);
CREATE INDEX memecooking_create_token_idx_timestamp_meme_id ON memecooking_create_token(timestamp, meme_id);
CREATE INDEX memecooking_create_token_idx_timestamp_token_id ON memecooking_create_token(timestamp, token_id);
CREATE INDEX memecooking_create_token_testnet_idx_timestamp_meme_id ON memecooking_create_token_testnet(timestamp, meme_id);
CREATE INDEX memecooking_create_token_testnet_idx_timestamp_token_id ON memecooking_create_token_testnet(timestamp, token_id);
CREATE INDEX trade_pool_testnet_idx_timestamp_trader ON trade_pool_testnet(timestamp, trader);
CREATE INDEX trade_pool_testnet_idx_timestamp_pool ON trade_pool_testnet(timestamp, pool);
CREATE INDEX trade_pool_testnet_idx_timestamp_token_in ON trade_pool_testnet(timestamp, token_in);
CREATE INDEX trade_pool_testnet_idx_timestamp_token_out ON trade_pool_testnet(timestamp, token_out);
CREATE INDEX trade_swap_testnet_idx_timestamp_trader ON trade_swap_testnet(timestamp, trader);
CREATE INDEX trade_swap_testnet_idx_timestamp_balance_change_token ON trade_swap_testnet(timestamp, balance_changes);
CREATE INDEX trade_pool_change_testnet_idx_timestamp_pool_id ON trade_pool_change_testnet(timestamp, pool_id);
CREATE INDEX newcontract_nep141_testnet_idx_timestamp_account_id ON newcontract_nep141_testnet(timestamp, account_id);
CREATE INDEX tx_transactions_testnet_idx_timestamp_signer_id ON tx_transactions_testnet(timestamp, signer_id);
CREATE INDEX tx_transactions_testnet_idx_timestamp_receiver_id ON tx_transactions_testnet(timestamp, receiver_id);
CREATE INDEX tx_transactions_testnet_idx_timestamp_transaction_id ON tx_transactions_testnet(timestamp, transaction_id);
CREATE INDEX tx_transactions_testnet_idx_timestamp_public_key ON tx_transactions_testnet(timestamp, public_key);
CREATE INDEX tx_transactions_idx_timestamp_signer_id ON tx_transactions(timestamp, signer_id);
CREATE INDEX tx_transactions_idx_timestamp_receiver_id ON tx_transactions(timestamp, receiver_id);
CREATE INDEX tx_transactions_idx_timestamp_transaction_id ON tx_transactions(timestamp, transaction_id);
CREATE INDEX tx_transactions_idx_timestamp_public_key ON tx_transactions(timestamp, public_key);
CREATE INDEX tx_receipts_testnet_idx_timestamp_receipt_id ON tx_receipts_testnet(timestamp, receipt_id);
CREATE INDEX tx_receipts_testnet_idx_timestamp_transaction_id ON tx_receipts_testnet(timestamp, transaction_id);
CREATE INDEX tx_receipts_testnet_idx_timestamp_executor_id ON tx_receipts_testnet(timestamp, executor_id);
CREATE INDEX tx_receipts_idx_timestamp_receipt_id ON tx_receipts(timestamp, receipt_id);
CREATE INDEX tx_receipts_idx_timestamp_transaction_id ON tx_receipts(timestamp, transaction_id);
CREATE INDEX tx_receipts_idx_timestamp_executor_id ON tx_receipts(timestamp, executor_id);
