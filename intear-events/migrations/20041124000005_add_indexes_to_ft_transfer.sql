CREATE INDEX idx_ft_transfer_new_owner_id ON ft_transfer(new_owner_id, id DESC);
CREATE INDEX idx_ft_transfer_old_owner_id ON ft_transfer(old_owner_id, id DESC);
CREATE INDEX idx_ft_transfer_token_id ON ft_transfer(token_id, id DESC);
CREATE INDEX idx_ft_transfer_token_sender_id ON ft_transfer(token_id, old_owner_id, id DESC);
CREATE INDEX idx_ft_transfer_token_receiver_id ON ft_transfer(token_id, new_owner_id, id DESC); 