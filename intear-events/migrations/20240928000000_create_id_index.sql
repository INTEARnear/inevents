CREATE INDEX nft_transfer_idx_id ON nft_transfer (id);
CREATE INDEX nft_mint_idx_id ON nft_mint (id);
CREATE INDEX nft_burn_idx_id ON nft_burn (id);
CREATE INDEX potlock_donation_idx_id ON potlock_donation (id);
CREATE INDEX potlock_pot_project_donation_idx_id ON potlock_pot_project_donation (id);
CREATE INDEX potlock_pot_donation_idx_id ON potlock_pot_donation (id);
CREATE INDEX trade_pool_idx_id ON trade_pool (id);
CREATE INDEX trade_swap_idx_id ON trade_swap (id);
CREATE INDEX trade_pool_change_idx_id ON trade_pool_change (id);
CREATE INDEX price_pool_idx_id ON price_pool (id);
CREATE INDEX price_token_idx_id ON price_token (id);
CREATE INDEX newcontract_nep141_idx_id ON newcontract_nep141 (id);
CREATE INDEX socialdb_index_idx_id ON socialdb_index (id);
CREATE INDEX log_nep297_idx_id ON log_nep297 (id);
CREATE INDEX log_text_idx_id ON log_text (id);
CREATE INDEX new_memecooking_meme_idx_id ON new_memecooking_meme (id);
CREATE INDEX new_memecooking_meme_testnet_idx_id ON new_memecooking_meme_testnet (id);
CREATE INDEX moretps_testnet_idx_id ON moretps_testnet (id);
CREATE INDEX moretps_idx_id ON moretps (id);
CREATE INDEX block_info_testnet_idx_id ON block_info_testnet (id);
CREATE INDEX block_info_idx_id ON block_info (id);
CREATE INDEX memecooking_deposit_idx_id ON memecooking_deposit (id);
CREATE INDEX memecooking_deposit_testnet_idx_id ON memecooking_deposit_testnet (id);
CREATE INDEX memecooking_withdraw_idx_id ON memecooking_withdraw (id);
CREATE INDEX memecooking_withdraw_testnet_idx_id ON memecooking_withdraw_testnet (id);
CREATE INDEX memecooking_create_token_idx_id ON memecooking_create_token (id);
CREATE INDEX memecooking_create_token_testnet_idx_id ON memecooking_create_token_testnet (id);
CREATE INDEX trade_pool_testnet_idx_id ON trade_pool_testnet (id);
CREATE INDEX trade_swap_testnet_idx_id ON trade_swap_testnet (id);
CREATE INDEX trade_pool_change_testnet_idx_id ON trade_pool_change_testnet (id);
CREATE INDEX newcontract_nep141_testnet_idx_id ON newcontract_nep141_testnet (id);
CREATE INDEX tx_transactions_testnet_idx_id ON tx_transactions_testnet (id);
CREATE INDEX tx_transactions_idx_id ON tx_transactions (id);
CREATE INDEX tx_receipts_testnet_idx_id ON tx_receipts_testnet (id);
CREATE INDEX tx_receipts_idx_id ON tx_receipts (id);

CREATE INDEX nft_transfer_idx_id_old_owner_id ON nft_transfer(id, old_owner_id);
CREATE INDEX nft_transfer_idx_id_new_owner_id ON nft_transfer(id, new_owner_id);
CREATE INDEX nft_transfer_idx_id_contract_id ON nft_transfer(id, contract_id);
CREATE INDEX nft_mint_idx_id_owner_id ON nft_mint(id, owner_id);
CREATE INDEX nft_mint_idx_id_contract_id ON nft_mint(id, contract_id);
CREATE INDEX nft_burn_idx_id_owner_id ON nft_burn(id, owner_id);
CREATE INDEX nft_burn_idx_id_contract_id ON nft_burn(id, contract_id);
CREATE INDEX potlock_donation_idx_id_donor_id ON potlock_donation(id, donor_id);
CREATE INDEX potlock_donation_idx_id_project_id ON potlock_donation(id, project_id);
CREATE INDEX potlock_donation_idx_id_referrer_id ON potlock_donation(id, referrer_id);
CREATE INDEX potlock_pot_project_donation_idx_id_pot_id ON potlock_pot_project_donation(id, pot_id);
CREATE INDEX potlock_pot_project_donation_idx_id_donor_id ON potlock_pot_project_donation(id, donor_id);
CREATE INDEX potlock_pot_project_donation_idx_id_project_id ON potlock_pot_project_donation(id, project_id);
CREATE INDEX potlock_pot_project_donation_idx_id_referrer_id ON potlock_pot_project_donation(id, referrer_id);
CREATE INDEX potlock_pot_donation_idx_id_pot_id ON potlock_pot_donation(id, pot_id);
CREATE INDEX potlock_pot_donation_idx_id_donor_id ON potlock_pot_donation(id, donor_id);
CREATE INDEX potlock_pot_donation_idx_id_referrer_id ON potlock_pot_donation(id, referrer_id);
CREATE INDEX trade_pool_idx_id_trader ON trade_pool(id, trader);
CREATE INDEX trade_pool_idx_id_pool ON trade_pool(id, pool);
CREATE INDEX trade_pool_idx_id_token_in ON trade_pool(id, token_in);
CREATE INDEX trade_pool_idx_id_token_out ON trade_pool(id, token_out);
CREATE INDEX trade_swap_idx_id_trader ON trade_swap(id, trader);
CREATE INDEX trade_swap_idx_id_balance_change_token ON trade_swap(id, balance_changes);
CREATE INDEX trade_pool_change_idx_id_pool_id ON trade_pool_change(id, pool_id);
CREATE INDEX price_pool_idx_id_pool_id ON price_pool(id, pool_id);
CREATE INDEX price_pool_idx_id_token0 ON price_pool(id, token0);
CREATE INDEX price_pool_idx_id_token1 ON price_pool(id, token1);
CREATE INDEX price_token_idx_id_token ON price_token(id, token);
CREATE INDEX newcontract_nep141_idx_id_account_id ON newcontract_nep141(id, account_id);
CREATE INDEX socialdb_index_idx_id_account_id ON socialdb_index(id, account_id);
CREATE INDEX socialdb_index_idx_id_index_type ON socialdb_index(id, index_type);
CREATE INDEX socialdb_index_idx_id_index_key ON socialdb_index(id, index_key);
CREATE INDEX log_nep297_idx_id_account_id ON log_nep297(id, account_id);
CREATE INDEX log_nep297_idx_id_predecessor_id ON log_nep297(id, predecessor_id);
CREATE INDEX log_nep297_idx_id_event_standard ON log_nep297(id, event_standard);
CREATE INDEX log_nep297_idx_id_event_version ON log_nep297(id, event_version);
CREATE INDEX log_nep297_idx_id_event_event ON log_nep297(id, event_event);
CREATE INDEX log_text_idx_id_account_id ON log_text(id, account_id);
CREATE INDEX log_text_idx_id_predecessor_id ON log_text(id, predecessor_id);
CREATE INDEX new_memecooking_meme_idx_id_meme_id ON new_memecooking_meme(id, meme_id);
CREATE INDEX new_memecooking_meme_idx_id_owner ON new_memecooking_meme(id, owner);
CREATE INDEX new_memecooking_meme_testnet_idx_id_meme_id ON new_memecooking_meme_testnet(id, meme_id);
CREATE INDEX new_memecooking_meme_testnet_idx_id_owner ON new_memecooking_meme_testnet(id, owner);
CREATE INDEX moretps_testnet_idx_id_claimed_account_id ON moretps_testnet(id, claimed_account_id);
CREATE INDEX moretps_testnet_idx_id_claimed_parent_account_id ON moretps_testnet(id, claimed_parent_account_id);
CREATE INDEX moretps_testnet_idx_id_round_account_id ON moretps_testnet(id, round_account_id);
CREATE INDEX moretps_testnet_idx_id_round_parent_account_id ON moretps_testnet(id, round_parent_account_id);
CREATE INDEX moretps_idx_id_claimed_account_id ON moretps(id, claimed_account_id);
CREATE INDEX moretps_idx_id_claimed_parent_account_id ON moretps(id, claimed_parent_account_id);
CREATE INDEX moretps_idx_id_round_account_id ON moretps(id, round_account_id);
CREATE INDEX moretps_idx_id_round_parent_account_id ON moretps(id, round_parent_account_id);
CREATE INDEX block_info_testnet_idx_id_block_height ON block_info_testnet(id, block_height);
CREATE INDEX block_info_testnet_idx_id_block_hash ON block_info_testnet(id, block_hash);
CREATE INDEX block_info_testnet_idx_id_block_producer ON block_info_testnet(id, block_producer);
CREATE INDEX block_info_idx_id_block_height ON block_info(id, block_height);
CREATE INDEX block_info_idx_id_block_hash ON block_info(id, block_hash);
CREATE INDEX block_info_idx_id_block_producer ON block_info(id, block_producer);
CREATE INDEX memecooking_deposit_idx_id_meme_id ON memecooking_deposit(id, meme_id);
CREATE INDEX memecooking_deposit_idx_id_trader ON memecooking_deposit(id, trader);
CREATE INDEX memecooking_deposit_testnet_idx_id_meme_id ON memecooking_deposit_testnet(id, meme_id);
CREATE INDEX memecooking_deposit_testnet_idx_id_trader ON memecooking_deposit_testnet(id, trader);
CREATE INDEX memecooking_withdraw_idx_id_meme_id ON memecooking_withdraw(id, meme_id);
CREATE INDEX memecooking_withdraw_idx_id_trader ON memecooking_withdraw(id, trader);
CREATE INDEX memecooking_withdraw_testnet_idx_id_meme_id ON memecooking_withdraw_testnet(id, meme_id);
CREATE INDEX memecooking_withdraw_testnet_idx_id_trader ON memecooking_withdraw_testnet(id, trader);
CREATE INDEX memecooking_create_token_idx_id_meme_id ON memecooking_create_token(id, meme_id);
CREATE INDEX memecooking_create_token_idx_id_token_id ON memecooking_create_token(id, token_id);
CREATE INDEX memecooking_create_token_testnet_idx_id_meme_id ON memecooking_create_token_testnet(id, meme_id);
CREATE INDEX memecooking_create_token_testnet_idx_id_token_id ON memecooking_create_token_testnet(id, token_id);
CREATE INDEX trade_pool_testnet_idx_id_trader ON trade_pool_testnet(id, trader);
CREATE INDEX trade_pool_testnet_idx_id_pool ON trade_pool_testnet(id, pool);
CREATE INDEX trade_pool_testnet_idx_id_token_in ON trade_pool_testnet(id, token_in);
CREATE INDEX trade_pool_testnet_idx_id_token_out ON trade_pool_testnet(id, token_out);
CREATE INDEX trade_swap_testnet_idx_id_trader ON trade_swap_testnet(id, trader);
CREATE INDEX trade_swap_testnet_idx_id_balance_change_token ON trade_swap_testnet(id, balance_changes);
CREATE INDEX trade_pool_change_testnet_idx_id_pool_id ON trade_pool_change_testnet(id, pool_id);
CREATE INDEX newcontract_nep141_testnet_idx_id_account_id ON newcontract_nep141_testnet(id, account_id);
CREATE INDEX tx_transactions_testnet_idx_id_signer_id ON tx_transactions_testnet(id, signer_id);
CREATE INDEX tx_transactions_testnet_idx_id_receiver_id ON tx_transactions_testnet(id, receiver_id);
CREATE INDEX tx_transactions_testnet_idx_id_transaction_id ON tx_transactions_testnet(id, transaction_id);
CREATE INDEX tx_transactions_testnet_idx_id_public_key ON tx_transactions_testnet(id, public_key);
CREATE INDEX tx_transactions_idx_id_signer_id ON tx_transactions(id, signer_id);
CREATE INDEX tx_transactions_idx_id_receiver_id ON tx_transactions(id, receiver_id);
CREATE INDEX tx_transactions_idx_id_transaction_id ON tx_transactions(id, transaction_id);
CREATE INDEX tx_transactions_idx_id_public_key ON tx_transactions(id, public_key);
CREATE INDEX tx_receipts_testnet_idx_id_receipt_id ON tx_receipts_testnet(id, receipt_id);
CREATE INDEX tx_receipts_testnet_idx_id_transaction_id ON tx_receipts_testnet(id, transaction_id);
CREATE INDEX tx_receipts_testnet_idx_id_executor_id ON tx_receipts_testnet(id, executor_id);
CREATE INDEX tx_receipts_idx_id_receipt_id ON tx_receipts(id, receipt_id);
CREATE INDEX tx_receipts_idx_id_transaction_id ON tx_receipts(id, transaction_id);
CREATE INDEX tx_receipts_idx_id_executor_id ON tx_receipts(id, executor_id);
