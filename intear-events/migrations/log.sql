events3=# SELECT MIN(block_timestamp), MAX(block_timestamp) FROM transactions;
              min              |              max              
-------------------------------+-------------------------------
 2024-12-15 06:11:21.962046+02 | 2024-12-16 14:26:47.291436+02
(1 row)

events3=# CREATE TABLE new_table (LIKE transactions);
CREATE TABLE
events3=# SELECT create_hypertable('new_table', 'block_timestamp', 
    chunk_time_interval => INTERVAL '4 hours');
   create_hypertable    
------------------------
 (7,public,new_table,t)
(1 row)

events3=# ALTER TABLE new_table SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'signer_account_id,receiver_account_id',
    timescaledb.compress_orderby = 'id DESC'
);
ALTER TABLE
events3=# SELECT add_compression_policy('new_table', INTERVAL '4 hours');
 add_compression_policy 
------------------------
                   1001
(1 row)

events3=# SELECT enable_chunk_skipping('new_table', 'id');
 enable_chunk_skipping 
-----------------------
 (54,t)
(1 row)

events3=# INSERT INTO new_table
SELECT * FROM transactions;
INSERT 0 11468662
events3=# EXPLAIN ANALYZE SELECT * FROM transactions WHERE receiver_account_id = 'v2.ref-finance.near' AND id < 1000000000 ORDER BY id DESC LIMIT 50;
                                                                                                        QUERY PLAN                                                                     
                                   
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
-----------------------------------
 Limit  (cost=258540.43..258540.55 rows=50 width=257) (actual time=191.420..191.443 rows=50 loops=1)
   ->  Sort  (cost=258540.43..260661.79 rows=848545 width=257) (actual time=177.294..177.314 rows=50 loops=1)
         Sort Key: _hyper_5_20_chunk.id DESC
         Sort Method: top-N heapsort  Memory: 51kB
         ->  Append  (cost=225.36..230352.37 rows=848545 width=257) (actual time=1.138..173.318 rows=40177 loops=1)
               ->  Custom Scan (DecompressChunk) on _hyper_5_20_chunk  (cost=225.36..6309.94 rows=28000 width=255) (actual time=1.138..5.075 rows=1448 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_25_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_25_chunk  (cost=0.42..6309.94 rows=28 width=394) (actual 
time=1.129..4.795 rows=28 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_21_chunk  (cost=341.17..7505.68 rows=22000 width=255) (actual time=1.093..6.365 rows=1001 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_26_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_26_chunk  (cost=0.42..7505.68 rows=22 width=394) (actual 
time=1.090..6.088 rows=22 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_22_chunk  (cost=311.90..7485.63 rows=24000 width=255) (actual time=1.684..5.876 rows=978 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_27_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_27_chunk  (cost=0.42..7485.63 rows=24 width=394) (actual 
time=1.665..5.681 rows=22 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_23_chunk  (cost=255.61..6901.48 rows=27000 width=255) (actual time=0.939..5.293 rows=1045 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_28_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_28_chunk  (cost=0.42..6901.48 rows=27 width=394) (actual 
time=0.936..5.046 rows=27 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_24_chunk  (cost=346.00..9342.11 rows=27000 width=257) (actual time=1.058..6.916 rows=1090 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_29_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_29_chunk  (cost=0.42..9342.11 rows=27 width=395) (actual 
time=1.056..6.664 rows=28 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_30_chunk  (cost=277.01..7479.15 rows=27000 width=255) (actual time=1.183..5.852 rows=1170 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_38_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_38_chunk  (cost=0.42..7479.15 rows=27 width=394) (actual 
time=1.181..5.600 rows=26 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_31_chunk  (cost=276.65..7746.26 rows=28000 width=256) (actual time=1.333..5.816 rows=1065 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_39_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_39_chunk  (cost=0.42..7746.26 rows=28 width=394) (actual 
time=1.330..5.591 rows=29 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_32_chunk  (cost=343.16..7206.36 rows=21000 width=256) (actual time=1.224..5.261 rows=812 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_40_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_40_chunk  (cost=0.42..7206.36 rows=21 width=394) (actual 
time=1.222..5.074 rows=21 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_33_chunk  (cost=315.96..8214.88 rows=26000 width=255) (actual time=1.145..6.023 rows=1198 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_41_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_41_chunk  (cost=0.42..8214.88 rows=26 width=394) (actual 
time=1.143..5.779 rows=28 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_34_chunk  (cost=261.31..7316.71 rows=28000 width=257) (actual time=1.030..5.453 rows=1173 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_42_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_42_chunk  (cost=0.42..7316.71 rows=28 width=395) (actual 
time=1.028..5.188 rows=30 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_35_chunk  (cost=291.69..7583.85 rows=26000 width=256) (actual time=1.206..5.865 rows=891 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_47_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_47_chunk  (cost=0.42..7583.85 rows=26 width=394) (actual 
time=1.204..5.638 rows=26 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_36_chunk  (cost=283.27..7648.37 rows=27000 width=256) (actual time=1.016..5.756 rows=796 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_48_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_48_chunk  (cost=0.42..7648.37 rows=27 width=395) (actual 
time=1.013..5.561 rows=27 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_37_chunk  (cost=385.12..8087.55 rows=21000 width=256) (actual time=1.252..5.991 rows=836 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_49_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_49_chunk  (cost=0.42..8087.55 rows=21 width=394) (actual 
time=1.249..5.801 rows=22 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_43_chunk  (cost=328.15..7547.52 rows=23000 width=256) (actual time=1.511..5.580 rows=764 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_51_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_51_chunk  (cost=0.42..7547.52 rows=23 width=394) (actual 
time=1.493..5.409 rows=24 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_44_chunk  (cost=293.28..7625.22 rows=26000 width=256) (actual time=1.255..5.756 rows=1099 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_52_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_52_chunk  (cost=0.42..7625.22 rows=26 width=395) (actual 
time=1.253..5.536 rows=28 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_45_chunk  (cost=183.52..7340.76 rows=40000 width=256) (actual time=1.100..5.424 rows=1047 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_53_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_53_chunk  (cost=0.42..7340.76 rows=40 width=394) (actual 
time=1.097..5.170 rows=38 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_46_chunk  (cost=225.37..7437.29 rows=33000 width=256) (actual time=1.136..5.573 rows=1664 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_54_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_54_chunk  (cost=0.42..7437.29 rows=33 width=395) (actual 
time=1.134..5.280 rows=32 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_50_chunk  (cost=246.48..7147.87 rows=29000 width=256) (actual time=0.910..5.498 rows=1457 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_60_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_60_chunk  (cost=0.42..7147.87 rows=29 width=395) (actual 
time=0.908..5.206 rows=28 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_55_chunk  (cost=253.10..6327.44 rows=25000 width=256) (actual time=0.960..4.821 rows=1655 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_61_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_61_chunk  (cost=0.42..6327.44 rows=25 width=395) (actual 
time=0.946..4.526 rows=25 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_56_chunk  (cost=228.01..6384.18 rows=28000 width=257) (actual time=0.898..4.912 rows=1485 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_62_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_62_chunk  (cost=0.42..6384.18 rows=28 width=395) (actual 
time=0.895..4.641 rows=28 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_57_chunk  (cost=232.79..6750.78 rows=29000 width=268) (actual time=0.823..5.125 rows=1660 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_63_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_63_chunk  (cost=0.42..6750.78 rows=29 width=396) (actual 
time=0.808..4.825 rows=29 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_58_chunk  (cost=151.86..4707.57 rows=31000 width=268) (actual time=0.854..3.713 rows=1252 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_64_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_64_chunk  (cost=0.42..4707.57 rows=31 width=392) (actual 
time=0.851..3.438 rows=31 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_59_chunk  (cost=212.92..5961.77 rows=28000 width=268) (actual time=1.116..4.566 rows=1761 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_65_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_65_chunk  (cost=0.42..5961.77 rows=28 width=391) (actual 
time=1.113..4.226 rows=28 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_66_chunk  (cost=261.83..7331.32 rows=28000 width=268) (actual time=1.071..5.624 rows=1710 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_69_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_69_chunk  (cost=0.42..7331.32 rows=28 width=394) (actual 
time=1.068..5.291 rows=28 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_67_chunk  (cost=288.05..7489.18 rows=26000 width=268) (actual time=1.599..5.673 rows=1003 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_71_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_71_chunk  (cost=0.42..7489.18 rows=26 width=396) (actual 
time=1.575..5.433 rows=28 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_68_chunk  (cost=263.52..7115.00 rows=27000 width=268) (actual time=1.009..5.370 rows=1081 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_73_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_73_chunk  (cost=0.42..7115.00 rows=27 width=394) (actual 
time=1.007..5.112 rows=27 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_70_chunk  (cost=335.03..7370.77 rows=22000 width=268) (actual time=1.257..5.505 rows=1083 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_75_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_75_chunk  (cost=0.42..7370.77 rows=22 width=394) (actual 
time=1.232..5.272 rows=22 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_72_chunk  (cost=285.91..6861.93 rows=24000 width=256) (actual time=0.949..5.270 rows=1067 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_77_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_77_chunk  (cost=0.42..6861.93 rows=24 width=393) (actual 
time=0.946..5.031 rows=24 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_74_chunk  (cost=247.63..7181.35 rows=29000 width=257) (actual time=0.911..5.451 rows=1466 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_79_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_79_chunk  (cost=0.42..7181.35 rows=29 width=394) (actual 
time=0.909..5.165 rows=29 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_76_chunk  (cost=218.67..6997.41 rows=32000 width=256) (actual time=1.137..5.660 rows=1496 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_81_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_81_chunk  (cost=0.42..6997.41 rows=32 width=394) (actual 
time=1.135..5.361 rows=32 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_5_78_chunk  (cost=208.26..7080.89 rows=34000 width=257) (actual time=1.073..5.475 rows=1320 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_6_83_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_6_83_chunk  (cost=0.42..7080.89 rows=34 width=394) (actual 
time=1.071..5.205 rows=33 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Bitmap Heap Scan on _hyper_5_80_chunk  (cost=24.38..1226.59 rows=1157 width=256) (actual time=0.130..0.778 rows=1162 loops=1)
                     Recheck Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
                     Heap Blocks: exact=951
                     ->  Bitmap Index Scan on _hyper_5_80_chunk_transactions_receiver_account_id_id_idx  (cost=0.00..24.09 rows=1157 width=0) (actual time=0.071..0.071 rows=1162 loops
=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
               ->  Bitmap Heap Scan on _hyper_5_82_chunk  (cost=28.95..1396.82 rows=1388 width=257) (actual time=0.097..0.591 rows=1442 loops=1)
                     Recheck Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
                     Heap Blocks: exact=651
                     ->  Bitmap Index Scan on _hyper_5_82_chunk_transactions_receiver_account_id_id_idx  (cost=0.00..28.60 rows=1388 width=0) (actual time=0.061..0.061 rows=1445 loops
=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
 Planning Time: 2.745 ms
 JIT:
   Functions: 67
   Options: Inlining false, Optimization false, Expressions true, Deforming true
   Timing: Generation 1.339 ms, Inlining 0.000 ms, Optimization 0.555 ms, Emission 13.569 ms, Total 15.463 ms
 Execution Time: 193.035 ms
(145 rows)

events3=# EXPLAIN ANALYZE SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' AND id < 1000000000 ORDER BY id DESC LIMIT 50;
                                                                        QUERY PLAN                                                                         
-----------------------------------------------------------------------------------------------------------------------------------------------------------
 Limit  (cost=464239.82..464245.81 rows=50 width=256) (actual time=230.430..254.839 rows=50 loops=1)
   ->  Gather Merge  (cost=464239.82..469025.85 rows=39972 width=256) (actual time=225.540..249.946 rows=50 loops=1)
         Workers Planned: 4
         Workers Launched: 4
         ->  Sort  (cost=463239.76..463264.74 rows=9993 width=256) (actual time=217.611..217.615 rows=50 loops=5)
               Sort Key: _hyper_7_86_chunk.id DESC
               Sort Method: top-N heapsort  Memory: 76kB
               Worker 0:  Sort Method: top-N heapsort  Memory: 75kB
               Worker 1:  Sort Method: top-N heapsort  Memory: 51kB
               Worker 2:  Sort Method: top-N heapsort  Memory: 63kB
               Worker 3:  Sort Method: top-N heapsort  Memory: 51kB
               ->  Parallel Append  (cost=0.00..462907.80 rows=9993 width=256) (actual time=11.318..217.128 rows=8022 loops=5)
                     ->  Parallel Seq Scan on _hyper_7_86_chunk  (cost=0.00..62234.01 rows=1144 width=256) (actual time=15.626..161.714 rows=4058 loops=1)
                           Filter: ((id < 1000000000) AND (receiver_account_id = 'v2.ref-finance.near'::text))
                           Rows Removed by Filter: 1538837
                     ->  Parallel Seq Scan on _hyper_7_87_chunk  (cost=0.00..61919.51 rows=844 width=256) (actual time=15.719..148.545 rows=3746 loops=1)
                           Filter: ((id < 1000000000) AND (receiver_account_id = 'v2.ref-finance.near'::text))
                           Rows Removed by Filter: 1530496
                     ->  Parallel Seq Scan on _hyper_7_85_chunk  (cost=0.00..59942.92 rows=980 width=256) (actual time=10.869..136.670 rows=4137 loops=1)
                           Filter: ((id < 1000000000) AND (receiver_account_id = 'v2.ref-finance.near'::text))
                           Rows Removed by Filter: 1484463
                     ->  Parallel Seq Scan on _hyper_7_84_chunk  (cost=0.00..56783.06 rows=965 width=255) (actual time=14.298..139.049 rows=4472 loops=1)
                           Filter: ((id < 1000000000) AND (receiver_account_id = 'v2.ref-finance.near'::text))
                           Rows Removed by Filter: 1407882
                     ->  Parallel Seq Scan on _hyper_7_90_chunk  (cost=0.00..56153.29 rows=1194 width=257) (actual time=5.103..63.028 rows=2117 loops=2)
                           Filter: ((id < 1000000000) AND (receiver_account_id = 'v2.ref-finance.near'::text))
                           Rows Removed by Filter: 693728
                     ->  Parallel Seq Scan on _hyper_7_91_chunk  (cost=0.00..55870.56 rows=1396 width=256) (actual time=2.079..30.079 rows=1089 loops=5)
                           Filter: ((id < 1000000000) AND (receiver_account_id = 'v2.ref-finance.near'::text))
                           Rows Removed by Filter: 275830
                     ->  Parallel Seq Scan on _hyper_7_88_chunk  (cost=0.00..55047.87 rows=1602 width=256) (actual time=3.041..54.187 rows=3130 loops=2)
                           Filter: ((id < 1000000000) AND (receiver_account_id = 'v2.ref-finance.near'::text))
                           Rows Removed by Filter: 678678
                     ->  Parallel Seq Scan on _hyper_7_89_chunk  (cost=0.00..46663.64 rows=1545 width=255) (actual time=4.791..92.894 rows=6383 loops=1)
                           Filter: ((id < 1000000000) AND (receiver_account_id = 'v2.ref-finance.near'::text))
                           Rows Removed by Filter: 1152293
                     ->  Parallel Seq Scan on _hyper_7_92_chunk  (cost=0.00..8242.98 rows=539 width=257) (actual time=0.074..20.406 rows=1373 loops=1)
                           Filter: ((id < 1000000000) AND (receiver_account_id = 'v2.ref-finance.near'::text))
                           Rows Removed by Filter: 190624
 Planning Time: 1.199 ms
 JIT:
   Functions: 91
   Options: Inlining false, Optimization false, Expressions true, Deforming true
   Timing: Generation 2.491 ms, Inlining 0.000 ms, Optimization 1.424 ms, Emission 24.650 ms, Total 28.565 ms
 Execution Time: 255.537 ms
(45 rows)

events3=# SELECT compress_chunk(show_chunks('new_table'));
WARNING:  no index on "id" found for column range on chunk "_hyper_7_84_chunk"
DETAIL:  column range works best with an index on the dimension.
^CCancel request sent
ERROR:  canceling statement due to user request
events3=# ^C
events3=# CREATE INDEX new_table_id_idx ON new_table (id DESC);
CREATE INDEX new_table_signer_account_id_id_idx ON new_table (signer_account_id, id DESC);
CREATE INDEX new_table_receiver_account_id_id_idx ON new_table (receiver_account_id, id DESC);
CREATE INDEX new_table_block_timestamp_index_in_chunk_idx ON new_table (block_timestamp DESC, index_in_chunk DESC);
CREATE INDEX new_table_converted_into_receipt_id_idx ON new_table (converted_into_receipt_id DESC);
CREATE INDEX new_table_included_in_block_hash_idx ON new_table (included_in_block_hash DESC, id DESC);
CREATE INDEX new_table_included_in_chunk_hash_idx ON new_table (included_in_chunk_hash DESC, id DESC);
CREATE INDEX
CREATE INDEX
CREATE INDEX
CREATE INDEX
CREATE INDEX
CREATE INDEX
CREATE INDEX
events3=# EXPLAIN ANALYZE SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' AND id < 1000000000 ORDER BY id DESC LIMIT 50;
                                                                                          QUERY PLAN                                                                                   
        
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
--------
 Limit  (cost=3.98..58.59 rows=50 width=256) (actual time=0.081..0.132 rows=50 loops=1)
   ->  Merge Append  (cost=3.98..43661.91 rows=39977 width=256) (actual time=0.080..0.129 rows=50 loops=1)
         Sort Key: _hyper_7_84_chunk.id DESC
         ->  Index Scan using _hyper_7_84_chunk_new_table_receiver_account_id_id_idx on _hyper_7_84_chunk  (cost=0.43..4191.94 rows=3860 width=255) (actual time=0.011..0.011 rows=1 lo
ops=1)
               Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
         ->  Index Scan using _hyper_7_85_chunk_new_table_receiver_account_id_id_idx on _hyper_7_85_chunk  (cost=0.43..4261.89 rows=3920 width=256) (actual time=0.009..0.009 rows=1 lo
ops=1)
               Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
         ->  Index Scan using _hyper_7_86_chunk_new_table_receiver_account_id_id_idx on _hyper_7_86_chunk  (cost=0.43..4958.21 rows=4577 width=256) (actual time=0.008..0.008 rows=1 lo
ops=1)
               Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
         ->  Index Scan using _hyper_7_87_chunk_new_table_receiver_account_id_id_idx on _hyper_7_87_chunk  (cost=0.43..3688.54 rows=3375 width=256) (actual time=0.009..0.009 rows=1 lo
ops=1)
               Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
         ->  Index Scan using _hyper_7_88_chunk_new_table_receiver_account_id_id_idx on _hyper_7_88_chunk  (cost=0.43..6786.55 rows=6409 width=256) (actual time=0.010..0.010 rows=1 lo
ops=1)
               Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
         ->  Index Scan using _hyper_7_89_chunk_new_table_receiver_account_id_id_idx on _hyper_7_89_chunk  (cost=0.43..6487.76 rows=6180 width=255) (actual time=0.009..0.009 rows=1 lo
ops=1)
               Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
         ->  Index Scan using _hyper_7_90_chunk_new_table_receiver_account_id_id_idx on _hyper_7_90_chunk  (cost=0.43..5140.30 rows=4778 width=257) (actual time=0.007..0.007 rows=1 lo
ops=1)
               Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
         ->  Index Scan using _hyper_7_91_chunk_new_table_receiver_account_id_id_idx on _hyper_7_91_chunk  (cost=0.43..5973.50 rows=5585 width=256) (actual time=0.008..0.008 rows=1 lo
ops=1)
               Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
         ->  Index Scan using _hyper_7_92_chunk_new_table_receiver_account_id_id_idx on _hyper_7_92_chunk  (cost=0.42..1339.58 rows=1293 width=257) (actual time=0.008..0.054 rows=50 l
oops=1)
               Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (id < 1000000000))
 Planning Time: 1.681 ms
 Execution Time: 0.154 ms
(23 rows)

events3=# SELECT compress_chunk(show_chunks('new_table'));
             compress_chunk              
-----------------------------------------
 _timescaledb_internal._hyper_7_84_chunk
 _timescaledb_internal._hyper_7_85_chunk
 _timescaledb_internal._hyper_7_86_chunk
 _timescaledb_internal._hyper_7_87_chunk
 _timescaledb_internal._hyper_7_88_chunk
 _timescaledb_internal._hyper_7_89_chunk
 _timescaledb_internal._hyper_7_90_chunk
 _timescaledb_internal._hyper_7_91_chunk
 _timescaledb_internal._hyper_7_92_chunk
(9 rows)

events3=# EXPLAIN ANALYZE SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' AND id < 1000000000 ORDER BY id DESC LIMIT 50;
                                                                                                                  QUERY PLAN                                                           
                                                       
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
-------------------------------------------------------
 Limit  (cost=554713.08..554719.06 rows=50 width=256) (actual time=273.030..286.583 rows=50 loops=1)
   ->  Gather Merge  (cost=554713.08..3500508.52 rows=24602700 width=256) (actual time=186.982..200.532 rows=50 loops=1)
         Workers Planned: 4
         Workers Launched: 4
         ->  Sort  (cost=553713.02..569089.71 rows=6150675 width=256) (actual time=145.155..145.164 rows=40 loops=5)
               Sort Key: _hyper_7_92_chunk.id DESC
               Sort Method: top-N heapsort  Memory: 53kB
               Worker 0:  Sort Method: top-N heapsort  Memory: 55kB
               Worker 1:  Sort Method: top-N heapsort  Memory: 52kB
               Worker 2:  Sort Method: top-N heapsort  Memory: 53kB
               Worker 3:  Sort Method: quicksort  Memory: 25kB
               ->  Parallel Append  (cost=29.40..349392.02 rows=6150675 width=256) (actual time=26.444..144.322 rows=8022 loops=5)
                     ->  Custom Scan (DecompressChunk) on _hyper_7_92_chunk  (cost=163.73..21940.03 rows=134000 width=257) (actual time=83.632..88.073 rows=275 loops=5)
                           Vectorized Filter: (id < 1000000000)
                           ->  Parallel Seq Scan on compress_hyper_8_102_chunk  (cost=0.00..21940.03 rows=134 width=420) (actual time=83.631..88.001 rows=7 loops=5)
                                 Filter: ((_ts_meta_min_1 < 1000000000) AND (receiver_account_id = 'v2.ref-finance.near'::text))
                                 Rows Removed by Filter: 32740
                     ->  Custom Scan (DecompressChunk) on _hyper_7_87_chunk  (cost=29.48..39916.41 rows=1354000 width=256) (actual time=9.010..34.603 rows=3746 loops=1)
                           Vectorized Filter: (id < 1000000000)
                           ->  Parallel Index Scan using compress_hyper_8_97_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_97_chunk  (cost=0.55..39916.41 rows=1354 w
idth=420) (actual time=8.931..33.714 rows=60 loops=1)
                                 Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
                     ->  Custom Scan (DecompressChunk) on _hyper_7_86_chunk  (cost=29.45..40114.21 rows=1362000 width=256) (actual time=7.295..32.614 rows=4058 loops=1)
                           Vectorized Filter: (id < 1000000000)
                           ->  Parallel Index Scan using compress_hyper_8_96_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_96_chunk  (cost=0.55..40114.21 rows=1362 w
idth=420) (actual time=7.228..31.781 rows=57 loops=1)
                                 Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
                     ->  Custom Scan (DecompressChunk) on _hyper_7_85_chunk  (cost=29.40..38665.57 rows=1315000 width=256) (actual time=7.677..31.486 rows=4137 loops=1)
                           Vectorized Filter: (id < 1000000000)
                           ->  Parallel Index Scan using compress_hyper_8_95_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_95_chunk  (cost=0.43..38665.57 rows=1315 w
idth=420) (actual time=7.623..30.583 rows=52 loops=1)
                                 Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
                     ->  Custom Scan (DecompressChunk) on _hyper_7_88_chunk  (cost=20.90..35441.83 rows=1696000 width=256) (actual time=4.806..28.580 rows=6261 loops=1)
                           Vectorized Filter: (id < 1000000000)
                           ->  Parallel Index Scan using compress_hyper_8_98_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_98_chunk  (cost=0.43..35441.83 rows=1696 w
idth=420) (actual time=4.802..27.405 rows=50 loops=1)
                                 Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
                     ->  Custom Scan (DecompressChunk) on _hyper_7_90_chunk  (cost=20.89..37722.51 rows=1806000 width=257) (actual time=5.245..28.842 rows=4234 loops=1)
                           Vectorized Filter: (id < 1000000000)
                           ->  Parallel Index Scan using compress_hyper_8_100_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_100_chunk  (cost=0.43..37722.51 rows=1806 
width=420) (actual time=5.242..28.100 rows=39 loops=1)
                                 Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
                     ->  Custom Scan (DecompressChunk) on _hyper_7_91_chunk  (cost=20.86..37094.87 rows=1778000 width=256) (actual time=2.847..27.792 rows=1815 loops=3)
                           Vectorized Filter: (id < 1000000000)
                           ->  Parallel Index Scan using compress_hyper_8_101_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_101_chunk  (cost=0.43..37094.87 rows=1778 
width=420) (actual time=2.826..27.386 rows=18 loops=3)
                                 Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
                     ->  Custom Scan (DecompressChunk) on _hyper_7_89_chunk  (cost=20.80..30722.06 rows=1477000 width=255) (actual time=3.706..18.514 rows=6383 loops=1)
                           Vectorized Filter: (id < 1000000000)
                           ->  Parallel Index Scan using compress_hyper_8_99_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_99_chunk  (cost=0.43..30722.06 rows=1477 w
idth=420) (actual time=3.654..17.584 rows=47 loops=1)
                                 Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
                     ->  Custom Scan (DecompressChunk) on _hyper_7_84_chunk  (cost=20.80..37021.15 rows=1780000 width=255) (actual time=4.339..21.798 rows=4472 loops=1)
                           Vectorized Filter: (id < 1000000000)
                           ->  Parallel Index Scan using compress_hyper_8_94_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_94_chunk  (cost=0.43..37021.15 rows=1780 w
idth=420) (actual time=4.325..21.073 rows=47 loops=1)
                                 Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
 Planning Time: 2.067 ms
 JIT:
   Functions: 91
   Options: Inlining true, Optimization true, Expressions true, Deforming true
   Timing: Generation 2.217 ms, Inlining 86.394 ms, Optimization 231.557 ms, Emission 180.803 ms, Total 500.972 ms
 Execution Time: 287.224 ms
(55 rows)

events3=#    ANALYZE new_table;
ANALYZE
events3=# EXPLAIN ANALYZE SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' AND id < 1000000000 ORDER BY id DESC LIMIT 50;
                                                                                                         QUERY PLAN                                                                    
                                      
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
--------------------------------------
 Limit  (cost=241980.87..241981.00 rows=50 width=256) (actual time=178.782..178.791 rows=50 loops=1)
   ->  Sort  (cost=241980.87..243140.87 rows=464000 width=256) (actual time=173.922..173.929 rows=50 loops=1)
         Sort Key: _hyper_7_84_chunk.id DESC
         Sort Method: top-N heapsort  Memory: 53kB
         ->  Append  (cost=404.71..226567.13 rows=464000 width=256) (actual time=3.981..170.213 rows=40108 loops=1)
               ->  Custom Scan (DecompressChunk) on _hyper_7_84_chunk  (cost=404.71..27520.24 rows=68000 width=255) (actual time=3.981..20.794 rows=4472 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_94_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_94_chunk  (cost=0.43..27520.24 rows=68 width=394) (actual
 time=3.971..20.171 rows=47 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_85_chunk  (cost=574.50..28724.90 rows=50000 width=256) (actual time=4.145..22.120 rows=4137 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_95_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_95_chunk  (cost=0.43..28724.90 rows=50 width=394) (actual
 time=4.141..21.500 rows=52 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_86_chunk  (cost=584.59..29814.08 rows=51000 width=256) (actual time=3.852..21.682 rows=4058 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_96_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_96_chunk  (cost=0.55..29814.08 rows=51 width=394) (actual
 time=3.849..21.107 rows=57 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_87_chunk  (cost=462.83..29621.01 rows=64000 width=256) (actual time=4.540..21.708 rows=3746 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_97_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_97_chunk  (cost=0.55..29621.01 rows=64 width=394) (actual
 time=4.538..21.130 rows=60 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_88_chunk  (cost=497.22..26352.78 rows=53000 width=256) (actual time=3.577..19.622 rows=6261 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_98_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_98_chunk  (cost=0.43..26352.78 rows=53 width=395) (actual
 time=3.575..18.835 rows=50 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_89_chunk  (cost=495.50..22792.97 rows=46000 width=255) (actual time=3.486..17.386 rows=6383 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_99_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_99_chunk  (cost=0.42..22792.97 rows=46 width=394) (actual
 time=3.468..16.536 rows=47 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_90_chunk  (cost=637.21..28037.12 rows=44000 width=257) (actual time=3.824..20.861 rows=4234 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_100_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_100_chunk  (cost=0.43..28037.12 rows=44 width=394) (actua
l time=3.821..20.265 rows=39 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_91_chunk  (cost=500.91..27550.28 rows=55000 width=256) (actual time=3.780..21.286 rows=5444 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_101_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_101_chunk  (cost=0.43..27550.28 rows=55 width=394) (actua
l time=3.777..20.528 rows=53 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_92_chunk  (cost=116.17..3833.74 rows=33000 width=257) (actual time=0.567..3.376 rows=1373 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_102_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_102_chunk  (cost=0.42..3833.74 rows=33 width=396) (actual
 time=0.564..3.115 rows=33 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
 Planning Time: 1.857 ms
 JIT:
   Functions: 19
   Options: Inlining false, Optimization false, Expressions true, Deforming true
   Timing: Generation 0.640 ms, Inlining 0.000 ms, Optimization 0.248 ms, Emission 4.611 ms, Total 5.498 ms
 Execution Time: 179.535 ms
(47 rows)

events3=# EXPLAIN ANALYZE SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' AND id < 1000000000 ORDER BY id DESC LIMIT 50;
                                                                                                         QUERY PLAN                                                                    
                                      
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
--------------------------------------
 Limit  (cost=241980.87..241981.00 rows=50 width=256) (actual time=179.620..179.629 rows=50 loops=1)
   ->  Sort  (cost=241980.87..243140.87 rows=464000 width=256) (actual time=174.635..174.642 rows=50 loops=1)
         Sort Key: _hyper_7_84_chunk.id DESC
         Sort Method: top-N heapsort  Memory: 53kB
         ->  Append  (cost=404.71..226567.13 rows=464000 width=256) (actual time=3.998..170.929 rows=40108 loops=1)
               ->  Custom Scan (DecompressChunk) on _hyper_7_84_chunk  (cost=404.71..27520.24 rows=68000 width=255) (actual time=3.998..20.800 rows=4472 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_94_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_94_chunk  (cost=0.43..27520.24 rows=68 width=394) (actual
 time=3.984..20.176 rows=47 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_85_chunk  (cost=574.50..28724.90 rows=50000 width=256) (actual time=4.134..21.586 rows=4137 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_95_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_95_chunk  (cost=0.43..28724.90 rows=50 width=394) (actual
 time=4.131..21.003 rows=52 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_86_chunk  (cost=584.59..29814.08 rows=51000 width=256) (actual time=3.936..21.775 rows=4058 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_96_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_96_chunk  (cost=0.55..29814.08 rows=51 width=394) (actual
 time=3.933..21.206 rows=57 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_87_chunk  (cost=462.83..29621.01 rows=64000 width=256) (actual time=4.536..22.118 rows=3746 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_97_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_97_chunk  (cost=0.55..29621.01 rows=64 width=394) (actual
 time=4.533..21.535 rows=60 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_88_chunk  (cost=497.22..26352.78 rows=53000 width=256) (actual time=3.579..19.843 rows=6261 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_98_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_98_chunk  (cost=0.43..26352.78 rows=53 width=395) (actual
 time=3.575..19.040 rows=50 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_89_chunk  (cost=495.50..22792.97 rows=46000 width=255) (actual time=3.489..17.569 rows=6383 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_99_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_99_chunk  (cost=0.42..22792.97 rows=46 width=394) (actual
 time=3.472..16.717 rows=47 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_90_chunk  (cost=637.21..28037.12 rows=44000 width=257) (actual time=3.868..20.903 rows=4234 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_100_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_100_chunk  (cost=0.43..28037.12 rows=44 width=394) (actua
l time=3.865..20.280 rows=39 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_91_chunk  (cost=500.91..27550.28 rows=55000 width=256) (actual time=3.795..21.487 rows=5444 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_101_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_101_chunk  (cost=0.43..27550.28 rows=55 width=394) (actua
l time=3.792..20.732 rows=53 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
               ->  Custom Scan (DecompressChunk) on _hyper_7_92_chunk  (cost=116.17..3833.74 rows=33000 width=257) (actual time=0.571..3.468 rows=1373 loops=1)
                     Vectorized Filter: (id < 1000000000)
                     ->  Index Scan using compress_hyper_8_102_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_102_chunk  (cost=0.42..3833.74 rows=33 width=396) (actual
 time=0.568..3.214 rows=33 loops=1)
                           Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 1000000000))
 Planning Time: 0.923 ms
 JIT:
   Functions: 19
   Options: Inlining false, Optimization false, Expressions true, Deforming true
   Timing: Generation 0.630 ms, Inlining 0.000 ms, Optimization 0.270 ms, Emission 4.713 ms, Total 5.613 ms
 Execution Time: 180.371 ms
(47 rows)

events3=# EXPLAIN ANALYZE SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' ORDER BY id DESC LIMIT 50;
                                                                                                         QUERY PLAN                                                                    
                                      
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
--------------------------------------
 Limit  (cost=217803.45..217803.58 rows=50 width=256) (actual time=177.580..177.589 rows=50 loops=1)
   ->  Sort  (cost=217803.45..218963.45 rows=464000 width=256) (actual time=173.189..173.196 rows=50 loops=1)
         Sort Key: _hyper_7_84_chunk.id DESC
         Sort Method: top-N heapsort  Memory: 53kB
         ->  Append  (cost=360.85..202389.71 rows=464000 width=256) (actual time=4.365..169.472 rows=40108 loops=1)
               ->  Custom Scan (DecompressChunk) on _hyper_7_84_chunk  (cost=360.85..24537.59 rows=68000 width=255) (actual time=4.365..21.186 rows=4472 loops=1)
                     ->  Index Scan using compress_hyper_8_94_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_94_chunk  (cost=0.43..24537.59 rows=68 width=394) (actual
 time=4.355..20.529 rows=47 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_85_chunk  (cost=512.44..25622.02 rows=50000 width=256) (actual time=4.661..21.433 rows=4137 loops=1)
                     ->  Index Scan using compress_hyper_8_95_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_95_chunk  (cost=0.43..25622.02 rows=50 width=394) (actual
 time=4.658..20.827 rows=52 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_86_chunk  (cost=521.59..26601.31 rows=51000 width=256) (actual time=3.914..21.640 rows=4058 loops=1)
                     ->  Index Scan using compress_hyper_8_96_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_96_chunk  (cost=0.55..26601.31 rows=51 width=394) (actual
 time=3.912..21.064 rows=57 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_87_chunk  (cost=413.14..26440.79 rows=64000 width=256) (actual time=4.443..21.636 rows=3746 loops=1)
                     ->  Index Scan using compress_hyper_8_97_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_97_chunk  (cost=0.55..26440.79 rows=64 width=394) (actual
 time=4.440..21.064 rows=60 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_88_chunk  (cost=443.72..23517.15 rows=53000 width=256) (actual time=3.517..20.092 rows=6261 loops=1)
                     ->  Index Scan using compress_hyper_8_98_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_98_chunk  (cost=0.43..23517.15 rows=53 width=395) (actual
 time=3.515..19.263 rows=50 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_89_chunk  (cost=441.89..20327.09 rows=46000 width=255) (actual time=3.418..17.169 rows=6383 loops=1)
                     ->  Index Scan using compress_hyper_8_99_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_99_chunk  (cost=0.42..20327.09 rows=46 width=394) (actual
 time=3.399..16.326 rows=47 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_90_chunk  (cost=568.59..25017.86 rows=44000 width=257) (actual time=3.818..20.582 rows=4234 loops=1)
                     ->  Index Scan using compress_hyper_8_100_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_100_chunk  (cost=0.43..25017.86 rows=44 width=394) (actua
l time=3.816..19.986 rows=39 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_91_chunk  (cost=446.94..24581.49 rows=55000 width=256) (actual time=3.757..21.018 rows=5444 loops=1)
                     ->  Index Scan using compress_hyper_8_101_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_101_chunk  (cost=0.43..24581.49 rows=55 width=394) (actua
l time=3.755..20.264 rows=53 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_92_chunk  (cost=103.77..3424.41 rows=33000 width=257) (actual time=0.567..3.332 rows=1373 loops=1)
                     ->  Index Scan using compress_hyper_8_102_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_102_chunk  (cost=0.42..3424.41 rows=33 width=396) (actual
 time=0.565..3.074 rows=33 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
 Planning Time: 0.893 ms
 JIT:
   Functions: 19
   Options: Inlining false, Optimization false, Expressions true, Deforming true
   Timing: Generation 0.480 ms, Inlining 0.000 ms, Optimization 0.223 ms, Emission 4.166 ms, Total 4.869 ms
 Execution Time: 178.169 ms
(38 rows)

events3=# SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' ORDER BY id DESC LIMIT 50;
    id    |               transaction_hash               |            included_in_block_hash            |            included_in_chunk_hash            | index_in_chunk |      signer_a
ccount_id       | receiver_account_id |       status       |          converted_into_receipt_id           | receipt_conversion_gas_burnt | receipt_conversion_tokens_burnt |        blo
ck_timestamp        
----------+----------------------------------------------+----------------------------------------------+----------------------------------------------+----------------+--------------
----------------+---------------------+--------------------+----------------------------------------------+------------------------------+---------------------------------+-----------
--------------------
 11468459 | 9RM9o6UL24nBafzgYtKXuicXtoG5Ngx7DLZK6YPV8GHj | 9hQzB2VA5QbYzLiaHLT4bv24aWyQztwbzGCMgH3Jcchm | EfNV1PBo9LnXsM6CEdxX7TREYkusr5JZvE1GX6zXy3dq |              3 | aldor.near   
                | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BcbobLoFyUPgWsr4TrqDZpHCdr9Rb3AQY5sp63Xmm53H |                 335906789560 |            33590678956000000000 | 2024-12-16
 14:31:20.89851+02
 11468364 | FU9zB4wAMwGGqtc76aMTNgTun2LTTT6kLceLWUm3rLVA | FrMZ2pjTwZJduaeSHmuTgCNNGGzNG7Lwbb3kMhJC1S14 | BewW8pwtdqtNWtpcKCMKvN8xF5kfpVJzipLaCUjjG5sZ |              9 | aldor.near   
                | v2.ref-finance.near | SUCCESS_RECEIPT_ID | HVNwBZL2wgrHAYT6EZiZLvDn4bmuvLUhpXaDjMdjEfo2 |                 335906789560 |            33590678956000000000 | 2024-12-16
 14:31:19.728252+02
 11468342 | 5cuxbzouaX7YUapFjH8nq1JHENQfwTWjhzr6swMER7S9 | DTgC1S2BD66TJwjUK1kwHDeNugR452hEU8vSG13UGdm6 | 33N4T5qTJYXQTtau8y3nLtyd4LbtYiQHUkAFeuqW97oV |             23 | simpleguy.nea
r               | v2.ref-finance.near | SUCCESS_RECEIPT_ID | F6LCRMKVQUpB9nDy9fAaN9sE4Kky7oLNTJJQhSGZE51d |                 317977712720 |            31797771272000000000 | 2024-12-16
 14:31:18.483799+02
 11468211 | 41mkC8LjB8kPBjJyRqCyuiMYjpS62vRa9WXGVtVVzQkU | 8cQVwPSkcAYxMRB6hhbejjVkjYiVgd61eH49Zz8DE2rV | HWawVzK31maoRCe1GfXPriqsQivziNmWiqsBhXJuNeQy |              1 | trader-near.n
ear             | v2.ref-finance.near | SUCCESS_RECEIPT_ID | CsJHtr2NLqA3iRscTZuqFAs68sdeKyUaWh9c6Rd2uxsT |                 317977712720 |            31797771272000000000 | 2024-12-16
 14:31:17.230332+02
 11468066 | BLyBw7iiZeyUPtcGtURMTEJpjAPqi45xLuD1sibF5tPd | 8cQVwPSkcAYxMRB6hhbejjVkjYiVgd61eH49Zz8DE2rV | 8EZwLFEnUJUU7nAEQYBC16TT6cktJCwWvTipHrtbGWYa |             10 | aldor.near   
                | v2.ref-finance.near | SUCCESS_RECEIPT_ID | Hxw7bAC2hcHVA5UgBcX1E2PArfAm9CSeDD8Br4ut9Yjp |                 335906789560 |            33590678956000000000 | 2024-12-16
 14:31:17.230332+02
 11467796 | CCbsxReMBDWaGCcAaNjJf65HrKhJEEgTtfy9E7TC6s9a | 8vu4QN1MFysrKVXJYoXTZp8htSL4j2YMqKgyvkW5yi5P | 47pztvRvSi6HV7wc3bHyh9sgGX61UDTF2qfpMvjGe8Hy |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 8LT18Pn6Vt12AXqsouBahQQyukhH435WesKPD49h9FNX |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:31:13.382392+02
 11466998 | EX7tNbVVQ1zSEbxKFXon8pCqmDZPqnvjJ6Re5Mhy8vjT | DGETP2v5PDxoT9Mu7StKo79pwC9BfXuRan3uSx8TLH6z | DLEWimHreqetuGuyWsrwThG3abiKNcn8zmoKmHGMeXKN |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | E6mVfE3yAdwZJSVWun4yBQMJuaGjRrKj8VQSV3R1BkpR |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:31:05.662227+02
 11466533 | HgSZrr3yKww8XeWeLsV4ftfgv6vhWBayS7n36XxjL4H1 | Hm76HLokPAiogf1AMvuXV8wzTTrBW8k4RybEhLSZ2HL2 | 6myLLhRz958XU7mm1uYUx6fzzoZHPU4qZH2ZEREpLnsQ |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | HoPattSJtz2eLd6ku4zrsjTzcCGTr7t3cnC3Cm8b7P7R |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:31:00.089537+02
 11466258 | 4inTLGTavsmcuHbHv8txChke5QKQ5L8rCJHXQepdHxxp | EmpbVfA6u3dUkSpLsD4Pgx8FainZt9eH7KbTdYBnr4v  | 5yqojdKD5aJrWCCYWeX2NkiWahCKgNJA4LGPsLRWDxwG |              2 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 59g9fx9Roj4F1i7rh2gHUeGqBPy6BAAF33cXVrCz3n1U |                 310968206615 |            31096820661500000000 | 2024-12-16
 14:30:57.518861+02
 11465608 | ANj49gmT5tHUg3s7dFie5fReHmvimpdk82HUdcovdrrE | ByD6U1fSWNPtwjfyTiGLsonho55FZK37XrZtooPp1RBP | 6wkZb2SZjp3j4Dvn72TFMHgp9EDvuzZ5rsn8x34RHRWF |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 5HRPEXqV89eJoY1S9vu115UaTtsjUcuxR8qpGzbvwuhY |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:30:52.380317+02
 11464836 | Hqp4BbTThuoGni4qQNBSsKs9AvhqHWbcXnHykx2p3h1A | 72B2FbxmM3Tpx4iruzbCNjmKthjZ8bnWvmZ7H1Zi9tSJ | 3Frvk1RMRqRpmGrH1HAZaJX779qdDkC5JBNcbT2Q8km1 |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 6T9R5Wp6DntYVyKKic4Hqr1JxbMeQPoyVuwyRSpKF9DN |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:30:44.568748+02
 11464503 | 4v3JiKvC2Ps6hmpQajjDow9LwS85ED2HYMoMjzTDsHFG | 6bxJAa1wu6FjiF8ZJKZ8nCVqNNkeSYWyQs3DVTsnkyTU | 2ADWym2TGc8AT8Aa1FkyWpe8GBZ5xZdYGQhfxQq5Yyf4 |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 6LU7pxJ79o2FBPhH2YWRmLJbGZfYumbm2CuToXd6ASMi |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:30:41.874203+02
 11463854 | 9vtQHhvpox1MF5ykYZAc2Kxj35Ts3gLcAfmhY6tbQxox | J41bcnau1jWKDhRjj4tku177PhVCjQrKYGn4h7mFwiX  | Hat7uzfx4UpRiZCqf112Fkr1gfKn3eneigZ6bMk3FoNv |              0 | watcher02.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | FgcAtNqJXZQRpL8whAv2jEEGmWxCFFNYssk6Ptcq7at5 |                 312446401780 |            31244640178000000000 | 2024-12-16
 14:30:35.314694+02
 11463733 | DAEJisdxgtXv38DU8ggfmkfMej81aWWUy5QAkkxL3QkY | HAPqkwRXqdzMVhKbeZBJzHGyxVcSFYrRmPVZqLtrc1q1 | Hk9i7PT7FA8BFfp5TCGWESVYTCbVKKW2xJBeJyG6rVoP |              1 | watcher01.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | FFMvP1tiK3ha3WUUrJ51W5BDyS7x3KgZ3CcJV5QXw1Sa |                 312446401780 |            31244640178000000000 | 2024-12-16
 14:30:34.006976+02
 11463496 | 7E1m312QJ8j7K2FUbvEjcJmk3akgXVRCpRwfsEFNz1x7 | 8atz53zV5zNEec5scBT81k2eHpApjShVrtYkTq5YYYC7 | FKAYNTBfKttoS3Zezp8CRcwkZRdhpqfYYpA4sNQBpAAC |              2 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | D9apFnA7ZCAHUdhah1dLFFz2nzacBXfqhTQjuUs4DQf4 |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:30:31.345237+02
 11462739 | 2BEsKS79okp46J5AAJZPfP7kFYttvU7yja3LPmJShNxv | FSyzest6G3G1KpYy5t7ZSkJXLfAuZ2hGo2JN9xB1NHFX | Gnpa3yfBSFjTNqZWwqaJfs33CUaUb29kfZSY3KKMTwCN |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | Bsse8knD2UxrBH2r6T5Xpk4vnALLiGhUYLgvVgn9oKx4 |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:30:24.874347+02
 11460803 | BDj955UjzyMS26p7dwDchbcM8LQAuobWsVXcrVCjqQ8x | 3TNYc1CcSQNfGKrD743FaLUZxp68rrbe8BxipdQBvszx | HiHjB2LfrxFyvM8kFEsGwiPiiwvg4Qszn8GTq43TiP6C |              1 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BpR8uYVhdKa5K6bHCsXPobsKGkP3jAovDPayFMjmHmT7 |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:30:10.294821+02
 11460437 | 8FmXBKHdwin9ouoTTpPp1UWRP3Pv7CYrcZPkr4gP7DDG | HSRSFyaUgcYr5qWU4s7tGKhD99o1bJ97spkwCK3ZxXdB | 4ZD2fU7xwbKw7yL618XHFGVKYWyJozwSa89sZi44CnuQ |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 2DsTH9KizM4g6J457iPzthgpNhJxAeGmfmaVJVMpGRZB |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:30:06.344408+02
 11460195 | 8XtM8SM5TcQbhmtkJH3pw3F69uUtUuawVevWm7uSpET6 | 2S24CK59QzoALJosqeGsJ8HH9qYDtjMFsEj8UpTPJB9b | HTKcwA2Nzw5d4jwDWNkbPqeQHSZxmeNiFjYya7nWJzH1 |              2 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BAnyqLP6ZqNHCeqs8obFcfSQaDL26AsruyPQCEV82N9r |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:30:02.64026+02
 11459776 | HdP5jkLNLzUaZGw5gbnHyku222SQUvqCSN87DJBjD4Zq | G16JV6DtZr964ZrjG8TBx1LDU3YCHqhDBBx8q5fwBcqw | 5Lwu1dLtvWxsTiQgVZqnfUyovqMgV3tPfCV3ZkAxQJPp |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 56wDmW7HydPhZsDoquhKo5S8643JwLVu53TyzaySZ9qq |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:29:58.425705+02
 11459367 | DowPuFhZ9KDMqS98raTmGZob9rxveTyJtrSZt7jEXuFa | EtGUr6Czbc9sca4v2mveDKb9bF7Yr5rPE9LGDvL7y4kY | BtqwME5FJqh8Lacqq3aTr81cogeAmUUfu3wLF7jnriog |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | Ae67f3QexCHB7WtvQvavcH8yMEBhCdTRg8AMMmWVKZ9Y |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:29:54.471442+02
 11458995 | 9uLLwcm2EvjVh6RrawwYXqgvF9AQcK4m5Rz8512HwVP9 | 9ZyWLQ3YnsqkwSLhQjRuLdCkxQv52aWiYPk1SzVBxUi3 | 34cLANdMXcK2VHN3LJGq4cEFSu4YwvDZ9Rf9X3aTVCq4 |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 82fJHdZxHs1Cm8ExmsVvVVur6f33Z8r7uGZFSspp9vMz |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:29:51.819205+02
 11458242 | FBG9hQFvJor8yaZi7HqzjztgDuj61ZwW6ZGr3ZESpvPE | C91MP1eLppT3aypbazQN2qkhdKZWx3FVETt6FYf2CDo  | 4UiwQCcr6eYvt5GXbbZ8MXBEzkijvMSDRwAd3G8yAYMH |              4 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | AaYxw7L6u3MW4YoFbfZwjGmM57QHxzQg2L9kWpRTqJGu |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:29:45.241332+02
 11458057 | 9iS2KpxkDDdC2GDtYf3MdqAeeBy3xekQgAsczyDZ3tQe | BMDhnjN9VeconqdkbFESNuXC62Co59Mxfdm2NnFmBEXi | FE9MHHM7H496SYsZy8dVeBdEzM6TCXruw3QWaW1rDTzM |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 8XSexAoB61zCRcS53zAJGNLtvzbJ6TMQj5CUGBJpmatV |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:29:42.629661+02
 11457435 | 48zbpTRtHHC59dq5pMWkZMSjPjjGHNFearitvDXWFFwA | 618pWELYETcnBKEnH9BFAmiovNfDtjnCLv8B28bYUxLw | 138dbrCGHwuhKYpr6UX9wcdZJ1vJVRnAK9YoeJBmRuAR |              2 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | FzXJpYkZTvKmLrWpwXtDkVWY9ZYfrQYH4eUzdaYbFTE2 |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:29:37.391064+02
 11457205 | 8JypqxBJMhG9sXzySdtzLxjeTg5SJL8faN8RsL8H459y | 9akERx4RSwbdtJRmEfRpYPHR3TYqaPkZkgNS4akRhxGh | EcX7VQsKtjSS6Mv9TvwaBrqxBur5xXosfNC2LNUUZRQs |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BawEkveGXGD247h5HrS8vJKraobeL3DzF4iW8BDTTzaj |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:29:35.299133+02
 11457041 | CDLQc4oUexTR84HuGDVppM76pibX8oaqn8zojbPbQxL1 | evRrAAM1MhARRo6dxp1QaLA9jj7kRh3T67VeJUsMtNF  | HucHaZdbXdsMZXxgQQgMWQtoSYndE9rDymxU7wHudQ63 |             19 | sneaky_blob.u
ser.intear.near | v2.ref-finance.near | SUCCESS_RECEIPT_ID | DUNsH4xniwdd2nwFQ7Gcsd8feokCTFzMWtNP8pgE3bxe |                 312351034350 |            31235103435000000000 | 2024-12-16
 14:29:33.468454+02
 11456647 | BKiDdHmdMNaGPDoWUQXWFoHJ2MenDkmEBHviZGQEvEXe | A5UDchgaqohxY2B8vWP5CfahBKZbw3sMHn3f99j4umVv | XLZpBUa12DKhopjE7HAr7sV1Ko87demRmAU5MP6uZsd  |              3 | sneaky_blob.u
ser.intear.near | v2.ref-finance.near | SUCCESS_RECEIPT_ID | Dhhe5KWGJ6ZiBx6pSSAfH7379z6jLLyk7fPoZgvLoDri |                 316737936130 |            31673793613000000000 | 2024-12-16
 14:29:29.516222+02
 11456090 | ETGcSQTu4QaHgqkpzfDDsxVRSN7sspr1LR5vbZbhUzFp | AWXEuZ5bKGkbiFh8JhhMWjPYuy94ms4wjPkADxtMjMLP | CM9UmEVt8qD48sFGocFH6xr85dFWxCyoA314zPt7x7bK |              2 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | G35xd6sbsftbsZcnknCLC1siZ4vSWk8vwHddbySjENqv |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:29:24.248563+02
 11455892 | 5F1XRpQxd196nNnyD3DtskSnzAHrE6K8vkNaGfgNsZ2b | Czgj68jH9bCxVXwB9PEo3nCHtsYrz4uwbtESEGfJQdNk | 9M61PS5KyZitahXpgveonEdJ2cTZBovHTioaa7ZVJXp3 |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | CtbRBd7LqfvV3buERfNSzcv86nGrqkg3rJn1gAF2YTcB |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:29:22.905683+02
 11455182 | 98m7ZoP6MFVqA5at1EFP7De6Nogbi2au82QBRRYEEMHN | BWDYiB2ZVNryKFALk7X1jGaquUz6kyka2HGjeTkruiNX | AFSeLdD6KF7gMtYiwmU3VSPdeL5bqSWhWHTYo44wZCf8 |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 9H2goKrWHv3eyh4sDyFfG7sX7LF48uNRCerDue1EuKCg |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:29:14.964649+02
 11454855 | 3Bcikb5Rcx8uLfxpVFNoaWwUEw2G3iEUY8AGUvsuj5o4 | AcBMW7kBB4uama1aeceEw9RGWLHBd67WqE7j8SfJutJU | EhRnWzDAwoLNBP7s1eEWDPqaGf1wWDRa71iEyBw1awHx |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BeGQMbgn7qAt3cSNCRUhvTvFJbCZ9jf4eRqt5mZnuTMW |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:29:12.409175+02
 11454023 | E17sGxwbBjg3XtipYMU7DomE9vNPvnKnq1WJb15MMRV1 | 9Qbkmr6GPWmGg2XEeL5mk2tRMQJ46huPecDzPR7qqVwQ | H7PU6NTrNWZhA81SH4KptvyRXasirTTXbEtpQAt3xXKp |              1 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 6cpgC5hv82voDhvGSZs8hBDb2awnh28CrbQkwsMcpsn7 |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:29:04.469973+02
 11453319 | 3zPCgCR6oSUgvajdQtMGMKXHvwenMT6donU61AL2FhfR | HuQoSKP6DUvdMPb8CrCTrWxXU1kiwS52NdnSqjSvd8Lt | Bak1HCU6yN732a1XsxEPsU5T2UaynPxUFoyGcFjBNznr |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | DX72VPNGCuUqDsBVACJmacZaFpqc8asB9kJCJFwHiBpL |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:28:57.929157+02
 11453318 | 8vtKyCEHoi1ZyLmePa8WhR5iLNr7wdN86aiUH33eXS3W | HuQoSKP6DUvdMPb8CrCTrWxXU1kiwS52NdnSqjSvd8Lt | Bak1HCU6yN732a1XsxEPsU5T2UaynPxUFoyGcFjBNznr |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | HCUn2jaFdvNkbR2Mgxa2Vt3MwNBMe4kE96y2cgCVpfCp |                 310968206615 |            31096820661500000000 | 2024-12-16
 14:28:57.929157+02
 11452734 | AoBqhdcFoC9ySEK9AqFXU6ZRqEfJYdf8axQeG2mDBS7K | Ek9YzaHMVh3qtJbps537WSSNAfPU4wb3pFNdsVHhWeH5 | J5RLT9P8phad8md7D6Y2Y4yxswsAbzkt8UCQqLdbyNTs |              3 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 9XyCjh53oiheUSxirwJizVGJYR2ZnuWzMpBnNfFCW7RK |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:28:51.385023+02
 11452197 | A4k1rP2XMXwuPGXjduQKCbL6jvUEp3ESpAezBCamKHwL | NeKGDjWa6bKUyHjL7JZFgTPdpTiWL5Mtijq2vGsdRyH  | 5e8HcNDyQ9sSttASaQUKrmxfvRQJDi4M42GNp5FQyPv7 |              3 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 2eZCEKvEarFcXqaJiLHt7B7Sd6ALjcSEn4n2b5nT9B9p |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:28:44.753409+02
 11452196 | AGZuYAxeMrpQJwNKeg6XsCFXi2ncPzNyxVfHz6YSpXhx | NeKGDjWa6bKUyHjL7JZFgTPdpTiWL5Mtijq2vGsdRyH  | 5e8HcNDyQ9sSttASaQUKrmxfvRQJDi4M42GNp5FQyPv7 |              2 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | FWJsLEy6Wx44HrAKCT5DemvRiAu6xETWVbTAXRMGvVwe |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:28:44.753409+02
 11451023 | 8ahppvU6xhBhnv1jUUqRW7Jg6aQMTd4rJgUvF9PtXYrb | F5hbTdq9Ry3LmSsw7x3NL3z6eCA89TcW5fm4jrFbGeSw | 7xekqAcpECk9S3kutN3S2xPPNenhSNwvbAM6GjfyvxWZ |              7 | watcher01.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | fe5KuFwGRRaVFu29cAy1pSrBABrtWabVukSvvCUTtKa  |                 312446401780 |            31244640178000000000 | 2024-12-16
 14:28:33.20536+02
 11451020 | 2gjQE6q5ve5HGA3XcyXGrPoPt8Bvbqsrer6upsA71Xg9 | F5hbTdq9Ry3LmSsw7x3NL3z6eCA89TcW5fm4jrFbGeSw | 7xekqAcpECk9S3kutN3S2xPPNenhSNwvbAM6GjfyvxWZ |              4 | watcher02.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | DZPvw74cKrftxxcqV866nVg8FhgouCscof8RhTwZQjsK |                 312446401780 |            31244640178000000000 | 2024-12-16
 14:28:33.20536+02
 11450111 | Ew3jQfYEuLMpXD1YYvcNfXK9K6U7uSkSBhBwPr4UKB8f | 8NosEjCYwXK4Nng5NtgaPRowuHAdEm3samzeXLkBuJF  | ptstkyscLT3k4828w7AHSPvNP5EBbBmL2i4MrvdJYTF  |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 5xSkqWfvYa3LuJ4LFwd8L5iNxnZSW7ox4NP8A6Uf8H3c |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:28:24.998957+02
 11449920 | DL4LjpGxPdBbjdgs16XNeiQgqZz7icpykV9iZPYN3dZB | C1Qq4xPG8t6TuwunCSJE3p5cgWLiNanvk7w6VFpcD5W2 | 4k9rRAHyLVXntrNtEjszB6TaP2R7Q73M9ij3CkfekYBL |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | Lxr5Dne7tiE2yVibdrVt2QZaMMRrMwvZfxScz4YQJa9  |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:28:22.323234+02
 11449518 | 6mhzokSLZdGPMtVeoe3KnGnKrWpLs1XqLYujgpAJUZRb | ASXN9FrFJsnTzGjWnPkMPdxoth97FEiRSUmgQ7ahFykt | AP13La8K6r5wL7HJFZmka5MfQdpmS3hovsfG7Aa58vSa |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | EPZZ44ciZ6j3bPc4RpTvoY1xfWQEdoLWyPbUGKvBvsmM |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:28:18.396387+02
 11449203 | 8TiVBaS3rzUy1at7vubGwFV38kJnPx345MCqnby6tUsz | 4KHnN7JLWqXVfq5mxqnP4rcAjqdeA9kRCTEvDcZ3qTWm | HZ76dKca4hXhE1JdpGB86H7SmuQAWrpWYo3Mssas5AG1 |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BwgoXrEWtUAMiERA4Qn6at813DL2ERhyxBRbdWSZDuaq |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:28:15.72843+02
 11448201 | BETFQTJDtjXqMwbZjwKHjjaQWeHb941ZDfcdJVh7hwMC | CxGkXfc8xMYC4mYpuHiZViPhoFsLRvYAgJeovMFTYQFP | 66bZmw4tBVrgkMLczCDRoMiN5H1Lv24dRcXNW9WLuTak |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 43uVpXCK5oqFThFq8yK1tivAHGWdUFdztfVyBy7cAyea |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:28:06.526746+02
 11447368 | 5sHxYk2L2eVWJKQ5vP3VLG6FcpMShTYVz14buzqWGFJ1 | 7drR2Q5qn1jyrTPErCxguGuXeM54mSdYaRL3rFp3ipZK | Cn2Gy7N8Prkh8SNapVXQvS5hxeexpsCtUwJbDcNPnYPG |              2 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | AAXDmGDHpUmi7EQo8yzyAo1aLpp8wLENYp7VZCtNXZaT |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:27:58.567703+02
 11447367 | Evnu4nfRCPV8qGrYrGKzVMjFt3Xx43LgNMQs7NAT1Km7 | 7drR2Q5qn1jyrTPErCxguGuXeM54mSdYaRL3rFp3ipZK | Cn2Gy7N8Prkh8SNapVXQvS5hxeexpsCtUwJbDcNPnYPG |              1 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 47G61n9H7C29RmXV8oLK5pECUFftVp8k5Ff7GJRo1FRb |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:27:58.567703+02
 11446638 | Cth4nsRReU8urjr3Z7gEx4TKX6hnMncZ7FJqPFS77GjS | A1vGW1xNFftwiXTeVF5RXBafLQk5HkbLySM6Nw2iQ6YR | ACCZxEbmDjRUojJ9rJQBM8wBiZkKJR3KYi561DTx33hZ |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | YiFiUuFpM3hN8AtezLYSP22wP9nEaWEYVXudHnpHnxr  |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:27:51.937454+02
 11446422 | GcgynHstsSD1WCn4UpajVXw2qJXavVKKQUf5ayQnY2jx | 7rYsSnDWkcKsLMK1z5cCzdJzrEWQTqh8HTpQ65h8Gpg  | DqBPR84iho6uoPeSsq2ykTs6YSDW9XzgSLydcR9cYBoJ |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 997wxGrPP2fYUa9hGybVLcNS9tSxpRP1daPpgVLXnUrT |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:27:50.670198+02
 11445855 | 24VykH4YeMC8G7qGubRLkcfV1ciKmVniYANM59jLonPg | 68BKetJWxE7Gmj9TmktL9RQpLqNFb3dNKQdxonjBQyU  | G3itu1jmXKTzvfLPp4aDZ716EZe9VZWbncoygShskFXL |              3 | watcher01.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | A2hETfea3Br7rWiGecpNYpQvzNPs9APcQJsnDjKyRXGk |                 312446401780 |            31244640178000000000 | 2024-12-16
 14:27:47.743818+02
(50 rows)

events3=# SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' WHERE id < 11468342 AND id > 11445000 ORDER BY id DESC LIMIT 50;
ERROR:  syntax error at or near "WHERE"
LINE 1: ...WHERE receiver_account_id = 'v2.ref-finance.near' WHERE id <...
                                                             ^
events3=# SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' AND id < 11468342 AND id > 11445000 ORDER BY id DESC LIMIT 50;
    id    |               transaction_hash               |            included_in_block_hash            |            included_in_chunk_hash            | index_in_chunk |      signer_a
ccount_id       | receiver_account_id |       status       |          converted_into_receipt_id           | receipt_conversion_gas_burnt | receipt_conversion_tokens_burnt |        blo
ck_timestamp        
----------+----------------------------------------------+----------------------------------------------+----------------------------------------------+----------------+--------------
----------------+---------------------+--------------------+----------------------------------------------+------------------------------+---------------------------------+-----------
--------------------
 11468211 | 41mkC8LjB8kPBjJyRqCyuiMYjpS62vRa9WXGVtVVzQkU | 8cQVwPSkcAYxMRB6hhbejjVkjYiVgd61eH49Zz8DE2rV | HWawVzK31maoRCe1GfXPriqsQivziNmWiqsBhXJuNeQy |              1 | trader-near.n
ear             | v2.ref-finance.near | SUCCESS_RECEIPT_ID | CsJHtr2NLqA3iRscTZuqFAs68sdeKyUaWh9c6Rd2uxsT |                 317977712720 |            31797771272000000000 | 2024-12-16
 14:31:17.230332+02
 11468066 | BLyBw7iiZeyUPtcGtURMTEJpjAPqi45xLuD1sibF5tPd | 8cQVwPSkcAYxMRB6hhbejjVkjYiVgd61eH49Zz8DE2rV | 8EZwLFEnUJUU7nAEQYBC16TT6cktJCwWvTipHrtbGWYa |             10 | aldor.near   
                | v2.ref-finance.near | SUCCESS_RECEIPT_ID | Hxw7bAC2hcHVA5UgBcX1E2PArfAm9CSeDD8Br4ut9Yjp |                 335906789560 |            33590678956000000000 | 2024-12-16
 14:31:17.230332+02
 11467796 | CCbsxReMBDWaGCcAaNjJf65HrKhJEEgTtfy9E7TC6s9a | 8vu4QN1MFysrKVXJYoXTZp8htSL4j2YMqKgyvkW5yi5P | 47pztvRvSi6HV7wc3bHyh9sgGX61UDTF2qfpMvjGe8Hy |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 8LT18Pn6Vt12AXqsouBahQQyukhH435WesKPD49h9FNX |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:31:13.382392+02
 11466998 | EX7tNbVVQ1zSEbxKFXon8pCqmDZPqnvjJ6Re5Mhy8vjT | DGETP2v5PDxoT9Mu7StKo79pwC9BfXuRan3uSx8TLH6z | DLEWimHreqetuGuyWsrwThG3abiKNcn8zmoKmHGMeXKN |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | E6mVfE3yAdwZJSVWun4yBQMJuaGjRrKj8VQSV3R1BkpR |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:31:05.662227+02
 11466533 | HgSZrr3yKww8XeWeLsV4ftfgv6vhWBayS7n36XxjL4H1 | Hm76HLokPAiogf1AMvuXV8wzTTrBW8k4RybEhLSZ2HL2 | 6myLLhRz958XU7mm1uYUx6fzzoZHPU4qZH2ZEREpLnsQ |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | HoPattSJtz2eLd6ku4zrsjTzcCGTr7t3cnC3Cm8b7P7R |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:31:00.089537+02
 11466258 | 4inTLGTavsmcuHbHv8txChke5QKQ5L8rCJHXQepdHxxp | EmpbVfA6u3dUkSpLsD4Pgx8FainZt9eH7KbTdYBnr4v  | 5yqojdKD5aJrWCCYWeX2NkiWahCKgNJA4LGPsLRWDxwG |              2 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 59g9fx9Roj4F1i7rh2gHUeGqBPy6BAAF33cXVrCz3n1U |                 310968206615 |            31096820661500000000 | 2024-12-16
 14:30:57.518861+02
 11465608 | ANj49gmT5tHUg3s7dFie5fReHmvimpdk82HUdcovdrrE | ByD6U1fSWNPtwjfyTiGLsonho55FZK37XrZtooPp1RBP | 6wkZb2SZjp3j4Dvn72TFMHgp9EDvuzZ5rsn8x34RHRWF |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 5HRPEXqV89eJoY1S9vu115UaTtsjUcuxR8qpGzbvwuhY |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:30:52.380317+02
 11464836 | Hqp4BbTThuoGni4qQNBSsKs9AvhqHWbcXnHykx2p3h1A | 72B2FbxmM3Tpx4iruzbCNjmKthjZ8bnWvmZ7H1Zi9tSJ | 3Frvk1RMRqRpmGrH1HAZaJX779qdDkC5JBNcbT2Q8km1 |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 6T9R5Wp6DntYVyKKic4Hqr1JxbMeQPoyVuwyRSpKF9DN |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:30:44.568748+02
 11464503 | 4v3JiKvC2Ps6hmpQajjDow9LwS85ED2HYMoMjzTDsHFG | 6bxJAa1wu6FjiF8ZJKZ8nCVqNNkeSYWyQs3DVTsnkyTU | 2ADWym2TGc8AT8Aa1FkyWpe8GBZ5xZdYGQhfxQq5Yyf4 |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 6LU7pxJ79o2FBPhH2YWRmLJbGZfYumbm2CuToXd6ASMi |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:30:41.874203+02
 11463854 | 9vtQHhvpox1MF5ykYZAc2Kxj35Ts3gLcAfmhY6tbQxox | J41bcnau1jWKDhRjj4tku177PhVCjQrKYGn4h7mFwiX  | Hat7uzfx4UpRiZCqf112Fkr1gfKn3eneigZ6bMk3FoNv |              0 | watcher02.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | FgcAtNqJXZQRpL8whAv2jEEGmWxCFFNYssk6Ptcq7at5 |                 312446401780 |            31244640178000000000 | 2024-12-16
 14:30:35.314694+02
 11463733 | DAEJisdxgtXv38DU8ggfmkfMej81aWWUy5QAkkxL3QkY | HAPqkwRXqdzMVhKbeZBJzHGyxVcSFYrRmPVZqLtrc1q1 | Hk9i7PT7FA8BFfp5TCGWESVYTCbVKKW2xJBeJyG6rVoP |              1 | watcher01.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | FFMvP1tiK3ha3WUUrJ51W5BDyS7x3KgZ3CcJV5QXw1Sa |                 312446401780 |            31244640178000000000 | 2024-12-16
 14:30:34.006976+02
 11463496 | 7E1m312QJ8j7K2FUbvEjcJmk3akgXVRCpRwfsEFNz1x7 | 8atz53zV5zNEec5scBT81k2eHpApjShVrtYkTq5YYYC7 | FKAYNTBfKttoS3Zezp8CRcwkZRdhpqfYYpA4sNQBpAAC |              2 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | D9apFnA7ZCAHUdhah1dLFFz2nzacBXfqhTQjuUs4DQf4 |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:30:31.345237+02
 11462739 | 2BEsKS79okp46J5AAJZPfP7kFYttvU7yja3LPmJShNxv | FSyzest6G3G1KpYy5t7ZSkJXLfAuZ2hGo2JN9xB1NHFX | Gnpa3yfBSFjTNqZWwqaJfs33CUaUb29kfZSY3KKMTwCN |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | Bsse8knD2UxrBH2r6T5Xpk4vnALLiGhUYLgvVgn9oKx4 |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:30:24.874347+02
 11460803 | BDj955UjzyMS26p7dwDchbcM8LQAuobWsVXcrVCjqQ8x | 3TNYc1CcSQNfGKrD743FaLUZxp68rrbe8BxipdQBvszx | HiHjB2LfrxFyvM8kFEsGwiPiiwvg4Qszn8GTq43TiP6C |              1 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BpR8uYVhdKa5K6bHCsXPobsKGkP3jAovDPayFMjmHmT7 |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:30:10.294821+02
 11460437 | 8FmXBKHdwin9ouoTTpPp1UWRP3Pv7CYrcZPkr4gP7DDG | HSRSFyaUgcYr5qWU4s7tGKhD99o1bJ97spkwCK3ZxXdB | 4ZD2fU7xwbKw7yL618XHFGVKYWyJozwSa89sZi44CnuQ |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 2DsTH9KizM4g6J457iPzthgpNhJxAeGmfmaVJVMpGRZB |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:30:06.344408+02
 11460195 | 8XtM8SM5TcQbhmtkJH3pw3F69uUtUuawVevWm7uSpET6 | 2S24CK59QzoALJosqeGsJ8HH9qYDtjMFsEj8UpTPJB9b | HTKcwA2Nzw5d4jwDWNkbPqeQHSZxmeNiFjYya7nWJzH1 |              2 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BAnyqLP6ZqNHCeqs8obFcfSQaDL26AsruyPQCEV82N9r |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:30:02.64026+02
 11459776 | HdP5jkLNLzUaZGw5gbnHyku222SQUvqCSN87DJBjD4Zq | G16JV6DtZr964ZrjG8TBx1LDU3YCHqhDBBx8q5fwBcqw | 5Lwu1dLtvWxsTiQgVZqnfUyovqMgV3tPfCV3ZkAxQJPp |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 56wDmW7HydPhZsDoquhKo5S8643JwLVu53TyzaySZ9qq |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:29:58.425705+02
 11459367 | DowPuFhZ9KDMqS98raTmGZob9rxveTyJtrSZt7jEXuFa | EtGUr6Czbc9sca4v2mveDKb9bF7Yr5rPE9LGDvL7y4kY | BtqwME5FJqh8Lacqq3aTr81cogeAmUUfu3wLF7jnriog |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | Ae67f3QexCHB7WtvQvavcH8yMEBhCdTRg8AMMmWVKZ9Y |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:29:54.471442+02
 11458995 | 9uLLwcm2EvjVh6RrawwYXqgvF9AQcK4m5Rz8512HwVP9 | 9ZyWLQ3YnsqkwSLhQjRuLdCkxQv52aWiYPk1SzVBxUi3 | 34cLANdMXcK2VHN3LJGq4cEFSu4YwvDZ9Rf9X3aTVCq4 |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 82fJHdZxHs1Cm8ExmsVvVVur6f33Z8r7uGZFSspp9vMz |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:29:51.819205+02
 11458242 | FBG9hQFvJor8yaZi7HqzjztgDuj61ZwW6ZGr3ZESpvPE | C91MP1eLppT3aypbazQN2qkhdKZWx3FVETt6FYf2CDo  | 4UiwQCcr6eYvt5GXbbZ8MXBEzkijvMSDRwAd3G8yAYMH |              4 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | AaYxw7L6u3MW4YoFbfZwjGmM57QHxzQg2L9kWpRTqJGu |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:29:45.241332+02
 11458057 | 9iS2KpxkDDdC2GDtYf3MdqAeeBy3xekQgAsczyDZ3tQe | BMDhnjN9VeconqdkbFESNuXC62Co59Mxfdm2NnFmBEXi | FE9MHHM7H496SYsZy8dVeBdEzM6TCXruw3QWaW1rDTzM |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 8XSexAoB61zCRcS53zAJGNLtvzbJ6TMQj5CUGBJpmatV |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:29:42.629661+02
 11457435 | 48zbpTRtHHC59dq5pMWkZMSjPjjGHNFearitvDXWFFwA | 618pWELYETcnBKEnH9BFAmiovNfDtjnCLv8B28bYUxLw | 138dbrCGHwuhKYpr6UX9wcdZJ1vJVRnAK9YoeJBmRuAR |              2 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | FzXJpYkZTvKmLrWpwXtDkVWY9ZYfrQYH4eUzdaYbFTE2 |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:29:37.391064+02
 11457205 | 8JypqxBJMhG9sXzySdtzLxjeTg5SJL8faN8RsL8H459y | 9akERx4RSwbdtJRmEfRpYPHR3TYqaPkZkgNS4akRhxGh | EcX7VQsKtjSS6Mv9TvwaBrqxBur5xXosfNC2LNUUZRQs |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BawEkveGXGD247h5HrS8vJKraobeL3DzF4iW8BDTTzaj |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:29:35.299133+02
 11457041 | CDLQc4oUexTR84HuGDVppM76pibX8oaqn8zojbPbQxL1 | evRrAAM1MhARRo6dxp1QaLA9jj7kRh3T67VeJUsMtNF  | HucHaZdbXdsMZXxgQQgMWQtoSYndE9rDymxU7wHudQ63 |             19 | sneaky_blob.u
ser.intear.near | v2.ref-finance.near | SUCCESS_RECEIPT_ID | DUNsH4xniwdd2nwFQ7Gcsd8feokCTFzMWtNP8pgE3bxe |                 312351034350 |            31235103435000000000 | 2024-12-16
 14:29:33.468454+02
 11456647 | BKiDdHmdMNaGPDoWUQXWFoHJ2MenDkmEBHviZGQEvEXe | A5UDchgaqohxY2B8vWP5CfahBKZbw3sMHn3f99j4umVv | XLZpBUa12DKhopjE7HAr7sV1Ko87demRmAU5MP6uZsd  |              3 | sneaky_blob.u
ser.intear.near | v2.ref-finance.near | SUCCESS_RECEIPT_ID | Dhhe5KWGJ6ZiBx6pSSAfH7379z6jLLyk7fPoZgvLoDri |                 316737936130 |            31673793613000000000 | 2024-12-16
 14:29:29.516222+02
 11456090 | ETGcSQTu4QaHgqkpzfDDsxVRSN7sspr1LR5vbZbhUzFp | AWXEuZ5bKGkbiFh8JhhMWjPYuy94ms4wjPkADxtMjMLP | CM9UmEVt8qD48sFGocFH6xr85dFWxCyoA314zPt7x7bK |              2 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | G35xd6sbsftbsZcnknCLC1siZ4vSWk8vwHddbySjENqv |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:29:24.248563+02
 11455892 | 5F1XRpQxd196nNnyD3DtskSnzAHrE6K8vkNaGfgNsZ2b | Czgj68jH9bCxVXwB9PEo3nCHtsYrz4uwbtESEGfJQdNk | 9M61PS5KyZitahXpgveonEdJ2cTZBovHTioaa7ZVJXp3 |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | CtbRBd7LqfvV3buERfNSzcv86nGrqkg3rJn1gAF2YTcB |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:29:22.905683+02
 11455182 | 98m7ZoP6MFVqA5at1EFP7De6Nogbi2au82QBRRYEEMHN | BWDYiB2ZVNryKFALk7X1jGaquUz6kyka2HGjeTkruiNX | AFSeLdD6KF7gMtYiwmU3VSPdeL5bqSWhWHTYo44wZCf8 |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 9H2goKrWHv3eyh4sDyFfG7sX7LF48uNRCerDue1EuKCg |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:29:14.964649+02
 11454855 | 3Bcikb5Rcx8uLfxpVFNoaWwUEw2G3iEUY8AGUvsuj5o4 | AcBMW7kBB4uama1aeceEw9RGWLHBd67WqE7j8SfJutJU | EhRnWzDAwoLNBP7s1eEWDPqaGf1wWDRa71iEyBw1awHx |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BeGQMbgn7qAt3cSNCRUhvTvFJbCZ9jf4eRqt5mZnuTMW |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:29:12.409175+02
 11454023 | E17sGxwbBjg3XtipYMU7DomE9vNPvnKnq1WJb15MMRV1 | 9Qbkmr6GPWmGg2XEeL5mk2tRMQJ46huPecDzPR7qqVwQ | H7PU6NTrNWZhA81SH4KptvyRXasirTTXbEtpQAt3xXKp |              1 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 6cpgC5hv82voDhvGSZs8hBDb2awnh28CrbQkwsMcpsn7 |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:29:04.469973+02
 11453319 | 3zPCgCR6oSUgvajdQtMGMKXHvwenMT6donU61AL2FhfR | HuQoSKP6DUvdMPb8CrCTrWxXU1kiwS52NdnSqjSvd8Lt | Bak1HCU6yN732a1XsxEPsU5T2UaynPxUFoyGcFjBNznr |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | DX72VPNGCuUqDsBVACJmacZaFpqc8asB9kJCJFwHiBpL |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:28:57.929157+02
 11453318 | 8vtKyCEHoi1ZyLmePa8WhR5iLNr7wdN86aiUH33eXS3W | HuQoSKP6DUvdMPb8CrCTrWxXU1kiwS52NdnSqjSvd8Lt | Bak1HCU6yN732a1XsxEPsU5T2UaynPxUFoyGcFjBNznr |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | HCUn2jaFdvNkbR2Mgxa2Vt3MwNBMe4kE96y2cgCVpfCp |                 310968206615 |            31096820661500000000 | 2024-12-16
 14:28:57.929157+02
 11452734 | AoBqhdcFoC9ySEK9AqFXU6ZRqEfJYdf8axQeG2mDBS7K | Ek9YzaHMVh3qtJbps537WSSNAfPU4wb3pFNdsVHhWeH5 | J5RLT9P8phad8md7D6Y2Y4yxswsAbzkt8UCQqLdbyNTs |              3 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 9XyCjh53oiheUSxirwJizVGJYR2ZnuWzMpBnNfFCW7RK |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:28:51.385023+02
 11452197 | A4k1rP2XMXwuPGXjduQKCbL6jvUEp3ESpAezBCamKHwL | NeKGDjWa6bKUyHjL7JZFgTPdpTiWL5Mtijq2vGsdRyH  | 5e8HcNDyQ9sSttASaQUKrmxfvRQJDi4M42GNp5FQyPv7 |              3 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 2eZCEKvEarFcXqaJiLHt7B7Sd6ALjcSEn4n2b5nT9B9p |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:28:44.753409+02
 11452196 | AGZuYAxeMrpQJwNKeg6XsCFXi2ncPzNyxVfHz6YSpXhx | NeKGDjWa6bKUyHjL7JZFgTPdpTiWL5Mtijq2vGsdRyH  | 5e8HcNDyQ9sSttASaQUKrmxfvRQJDi4M42GNp5FQyPv7 |              2 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | FWJsLEy6Wx44HrAKCT5DemvRiAu6xETWVbTAXRMGvVwe |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:28:44.753409+02
 11451023 | 8ahppvU6xhBhnv1jUUqRW7Jg6aQMTd4rJgUvF9PtXYrb | F5hbTdq9Ry3LmSsw7x3NL3z6eCA89TcW5fm4jrFbGeSw | 7xekqAcpECk9S3kutN3S2xPPNenhSNwvbAM6GjfyvxWZ |              7 | watcher01.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | fe5KuFwGRRaVFu29cAy1pSrBABrtWabVukSvvCUTtKa  |                 312446401780 |            31244640178000000000 | 2024-12-16
 14:28:33.20536+02
 11451020 | 2gjQE6q5ve5HGA3XcyXGrPoPt8Bvbqsrer6upsA71Xg9 | F5hbTdq9Ry3LmSsw7x3NL3z6eCA89TcW5fm4jrFbGeSw | 7xekqAcpECk9S3kutN3S2xPPNenhSNwvbAM6GjfyvxWZ |              4 | watcher02.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | DZPvw74cKrftxxcqV866nVg8FhgouCscof8RhTwZQjsK |                 312446401780 |            31244640178000000000 | 2024-12-16
 14:28:33.20536+02
 11450111 | Ew3jQfYEuLMpXD1YYvcNfXK9K6U7uSkSBhBwPr4UKB8f | 8NosEjCYwXK4Nng5NtgaPRowuHAdEm3samzeXLkBuJF  | ptstkyscLT3k4828w7AHSPvNP5EBbBmL2i4MrvdJYTF  |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 5xSkqWfvYa3LuJ4LFwd8L5iNxnZSW7ox4NP8A6Uf8H3c |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:28:24.998957+02
 11449920 | DL4LjpGxPdBbjdgs16XNeiQgqZz7icpykV9iZPYN3dZB | C1Qq4xPG8t6TuwunCSJE3p5cgWLiNanvk7w6VFpcD5W2 | 4k9rRAHyLVXntrNtEjszB6TaP2R7Q73M9ij3CkfekYBL |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | Lxr5Dne7tiE2yVibdrVt2QZaMMRrMwvZfxScz4YQJa9  |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:28:22.323234+02
 11449518 | 6mhzokSLZdGPMtVeoe3KnGnKrWpLs1XqLYujgpAJUZRb | ASXN9FrFJsnTzGjWnPkMPdxoth97FEiRSUmgQ7ahFykt | AP13La8K6r5wL7HJFZmka5MfQdpmS3hovsfG7Aa58vSa |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | EPZZ44ciZ6j3bPc4RpTvoY1xfWQEdoLWyPbUGKvBvsmM |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:28:18.396387+02
 11449203 | 8TiVBaS3rzUy1at7vubGwFV38kJnPx345MCqnby6tUsz | 4KHnN7JLWqXVfq5mxqnP4rcAjqdeA9kRCTEvDcZ3qTWm | HZ76dKca4hXhE1JdpGB86H7SmuQAWrpWYo3Mssas5AG1 |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BwgoXrEWtUAMiERA4Qn6at813DL2ERhyxBRbdWSZDuaq |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:28:15.72843+02
 11448201 | BETFQTJDtjXqMwbZjwKHjjaQWeHb941ZDfcdJVh7hwMC | CxGkXfc8xMYC4mYpuHiZViPhoFsLRvYAgJeovMFTYQFP | 66bZmw4tBVrgkMLczCDRoMiN5H1Lv24dRcXNW9WLuTak |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 43uVpXCK5oqFThFq8yK1tivAHGWdUFdztfVyBy7cAyea |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:28:06.526746+02
 11447368 | 5sHxYk2L2eVWJKQ5vP3VLG6FcpMShTYVz14buzqWGFJ1 | 7drR2Q5qn1jyrTPErCxguGuXeM54mSdYaRL3rFp3ipZK | Cn2Gy7N8Prkh8SNapVXQvS5hxeexpsCtUwJbDcNPnYPG |              2 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | AAXDmGDHpUmi7EQo8yzyAo1aLpp8wLENYp7VZCtNXZaT |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:27:58.567703+02
 11447367 | Evnu4nfRCPV8qGrYrGKzVMjFt3Xx43LgNMQs7NAT1Km7 | 7drR2Q5qn1jyrTPErCxguGuXeM54mSdYaRL3rFp3ipZK | Cn2Gy7N8Prkh8SNapVXQvS5hxeexpsCtUwJbDcNPnYPG |              1 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 47G61n9H7C29RmXV8oLK5pECUFftVp8k5Ff7GJRo1FRb |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:27:58.567703+02
 11446638 | Cth4nsRReU8urjr3Z7gEx4TKX6hnMncZ7FJqPFS77GjS | A1vGW1xNFftwiXTeVF5RXBafLQk5HkbLySM6Nw2iQ6YR | ACCZxEbmDjRUojJ9rJQBM8wBiZkKJR3KYi561DTx33hZ |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | YiFiUuFpM3hN8AtezLYSP22wP9nEaWEYVXudHnpHnxr  |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:27:51.937454+02
 11446422 | GcgynHstsSD1WCn4UpajVXw2qJXavVKKQUf5ayQnY2jx | 7rYsSnDWkcKsLMK1z5cCzdJzrEWQTqh8HTpQ65h8Gpg  | DqBPR84iho6uoPeSsq2ykTs6YSDW9XzgSLydcR9cYBoJ |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 997wxGrPP2fYUa9hGybVLcNS9tSxpRP1daPpgVLXnUrT |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:27:50.670198+02
 11445855 | 24VykH4YeMC8G7qGubRLkcfV1ciKmVniYANM59jLonPg | 68BKetJWxE7Gmj9TmktL9RQpLqNFb3dNKQdxonjBQyU  | G3itu1jmXKTzvfLPp4aDZ716EZe9VZWbncoygShskFXL |              3 | watcher01.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | A2hETfea3Br7rWiGecpNYpQvzNPs9APcQJsnDjKyRXGk |                 312446401780 |            31244640178000000000 | 2024-12-16
 14:27:47.743818+02
 11445732 | FVY7rX78VzQeGmajX2fum5JSWiV1AcvQjEmDEnrcJLQF | HeVCTY6Udn6qaB2t5X5DGdwb97fUA8vAdyj3jd6JFphc | GxvLU9TJGQDV4d7r5d5WtnkYbih4NQWoPSgfBJ6TxCiX |              1 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 8ATz5Y2QTHYdxMGTJFvYbkrhYizn6TvimxzLZbEr1M6L |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:27:44.192452+02
 11445731 | 7pxnX5iAYVv9AeXo4wZtBK6hqJ38J9xZbKVDPVTQVM9i | HeVCTY6Udn6qaB2t5X5DGdwb97fUA8vAdyj3jd6JFphc | GxvLU9TJGQDV4d7r5d5WtnkYbih4NQWoPSgfBJ6TxCiX |              0 | watcher02.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 4Ee1f7Cx7qBvWtwk128vUjUNM8fkYVswMX1bJWEtAMke |                 312446401780 |            31244640178000000000 | 2024-12-16
 14:27:44.192452+02
 11445541 | 3QjBGUbFgLbx1SmNe7fiATeg7bWxTwGpPZybGz5ewFpQ | DFKrmS9f6bdBxuFR15ccdXjQiwgT6rMirMvcrn7AeaHY | Bk1GrzznwjKSCmLGYNaGcbYPH5T8G3Vt5TdERJAckbBR |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | CTsxs5E26THZG5cqVezvbCHEagNRJUxFEdK2X4WHfRQW |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:27:42.089032+02
(50 rows)

events3=# EXPLAIN ANALYZE SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' AND id < 11468342 AND id > 11445000 ORDER BY id DESC LIMIT 50;
                                                                                                    QUERY PLAN                                                                         
                            
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
----------------------------
 Limit  (cost=4226.63..4227.84 rows=50 width=257) (actual time=3.411..3.475 rows=50 loops=1)
   ->  Custom Scan (DecompressChunk) on _hyper_7_92_chunk  (cost=4226.63..4323.20 rows=4000 width=257) (actual time=3.411..3.472 rows=50 loops=1)
         Filter: ((id < 11468342) AND (id > 11445000))
         Rows Removed by Filter: 221
         ->  Sort  (cost=4226.62..4226.63 rows=4 width=396) (actual time=3.346..3.348 rows=8 loops=1)
               Sort Key: compress_hyper_8_102_chunk._ts_meta_max_1 DESC
               Sort Method: quicksort  Memory: 29kB
               ->  Index Scan using compress_hyper_8_102_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_102_chunk  (cost=0.42..4226.58 rows=4 width=396) (actual time=0
.843..3.341 rows=8 loops=1)
                     Index Cond: ((receiver_account_id = 'v2.ref-finance.near'::text) AND (_ts_meta_min_1 < 11468342) AND (_ts_meta_max_1 > 11445000))
 Planning Time: 0.374 ms
 Execution Time: 3.493 ms
(11 rows)

events3=# 8342^C
events3=# SELECT * FROM new_table^C
events3=# SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' ORDER BY id DESC LIMIT 50;
    id    |               transaction_hash               |            included_in_block_hash            |            included_in_chunk_hash            | index_in_chunk |      signer_a
ccount_id       | receiver_account_id |       status       |          converted_into_receipt_id           | receipt_conversion_gas_burnt | receipt_conversion_tokens_burnt |        blo
ck_timestamp        
----------+----------------------------------------------+----------------------------------------------+----------------------------------------------+----------------+--------------
----------------+---------------------+--------------------+----------------------------------------------+------------------------------+---------------------------------+-----------
--------------------
 11468459 | 9RM9o6UL24nBafzgYtKXuicXtoG5Ngx7DLZK6YPV8GHj | 9hQzB2VA5QbYzLiaHLT4bv24aWyQztwbzGCMgH3Jcchm | EfNV1PBo9LnXsM6CEdxX7TREYkusr5JZvE1GX6zXy3dq |              3 | aldor.near   
                | v2.ref-finance.near | SUCCESS_RECEIPT_ID | BcbobLoFyUPgWsr4TrqDZpHCdr9Rb3AQY5sp63Xmm53H |                 335906789560 |            33590678956000000000 | 2024-12-16
 14:31:20.89851+02
 11468364 | FU9zB4wAMwGGqtc76aMTNgTun2LTTT6kLceLWUm3rLVA | FrMZ2pjTwZJduaeSHmuTgCNNGGzNG7Lwbb3kMhJC1S14 | BewW8pwtdqtNWtpcKCMKvN8xF5kfpVJzipLaCUjjG5sZ |              9 | aldor.near   
                | v2.ref-finance.near | SUCCESS_RECEIPT_ID | HVNwBZL2wgrHAYT6EZiZLvDn4bmuvLUhpXaDjMdjEfo2 |                 335906789560 |            33590678956000000000 | 2024-12-16
 14:31:19.728252+02
 11468342 | 5cuxbzouaX7YUapFjH8nq1JHENQfwTWjhzr6swMER7S9 | DTgC1S2BD66TJwjUK1kwHDeNugR452hEU8vSG13UGdm6 | 33N4T5qTJYXQTtau8y3nLtyd4LbtYiQHUkAFeuqW97oV |             23 | simpleguy.nea
r               | v2.ref-finance.near | SUCCESS_RECEIPT_ID | F6LCRMKVQUpB9nDy9fAaN9sE4Kky7oLNTJJQhSGZE51d |                 317977712720 |            31797771272000000000 | 2024-12-16
 14:31:18.483799+02
 11468211 | 41mkC8LjB8kPBjJyRqCyuiMYjpS62vRa9WXGVtVVzQkU | 8cQVwPSkcAYxMRB6hhbejjVkjYiVgd61eH49Zz8DE2rV | HWawVzK31maoRCe1GfXPriqsQivziNmWiqsBhXJuNeQy |              1 | trader-near.n
ear             | v2.ref-finance.near | SUCCESS_RECEIPT_ID | CsJHtr2NLqA3iRscTZuqFAs68sdeKyUaWh9c6Rd2uxsT |                 317977712720 |            31797771272000000000 | 2024-12-16
 14:31:17.230332+02
 11468066 | BLyBw7iiZeyUPtcGtURMTEJpjAPqi45xLuD1sibF5tPd | 8cQVwPSkcAYxMRB6hhbejjVkjYiVgd61eH49Zz8DE2rV | 8EZwLFEnUJUU7nAEQYBC16TT6cktJCwWvTipHrtbGWYa |             10 | aldor.near   
                | v2.ref-finance.near | SUCCESS_RECEIPT_ID | Hxw7bAC2hcHVA5UgBcX1E2PArfAm9CSeDD8Br4ut9Yjp |                 335906789560 |            33590678956000000000 | 2024-12-16
 14:31:17.230332+02
 11467796 | CCbsxReMBDWaGCcAaNjJf65HrKhJEEgTtfy9E7TC6s9a | 8vu4QN1MFysrKVXJYoXTZp8htSL4j2YMqKgyvkW5yi5P | 47pztvRvSi6HV7wc3bHyh9sgGX61UDTF2qfpMvjGe8Hy |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 8LT18Pn6Vt12AXqsouBahQQyukhH435WesKPD49h9FNX |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:31:13.382392+02
 11466998 | EX7tNbVVQ1zSEbxKFXon8pCqmDZPqnvjJ6Re5Mhy8vjT | DGETP2v5PDxoT9Mu7StKo79pwC9BfXuRan3uSx8TLH6z | DLEWimHreqetuGuyWsrwThG3abiKNcn8zmoKmHGMeXKN |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | E6mVfE3yAdwZJSVWun4yBQMJuaGjRrKj8VQSV3R1BkpR |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:31:05.662227+02
 11466533 | HgSZrr3yKww8XeWeLsV4ftfgv6vhWBayS7n36XxjL4H1 | Hm76HLokPAiogf1AMvuXV8wzTTrBW8k4RybEhLSZ2HL2 | 6myLLhRz958XU7mm1uYUx6fzzoZHPU4qZH2ZEREpLnsQ |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | HoPattSJtz2eLd6ku4zrsjTzcCGTr7t3cnC3Cm8b7P7R |                 310205267175 |            31020526717500000000 | 2024-12-16
 14:31:00.089537+02
 11466258 | 4inTLGTavsmcuHbHv8txChke5QKQ5L8rCJHXQepdHxxp | EmpbVfA6u3dUkSpLsD4Pgx8FainZt9eH7KbTdYBnr4v  | 5yqojdKD5aJrWCCYWeX2NkiWahCKgNJA4LGPsLRWDxwG |              2 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 59g9fx9Roj4F1i7rh2gHUeGqBPy6BAAF33cXVrCz3n1U |                 310968206615 |            31096820661500000000 | 2024-12-16
 14:30:57.518861+02
 11465608 | ANj49gmT5tHUg3s7dFie5fReHmvimpdk82HUdcovdrrE | ByD6U1fSWNPtwjfyTiGLsonho55FZK37XrZtooPp1RBP | 6wkZb2SZjp3j4Dvn72TFMHgp9EDvuzZ5rsn8x34RHRWF |              1 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 5HRPEXqV89eJoY1S9vu115UaTtsjUcuxR8qpGzbvwuhY |                 312970922645 |            31297092264500000000 | 2024-12-16
 14:30:52.380317+02
 11464836 | Hqp4BbTThuoGni4qQNBSsKs9AvhqHWbcXnHykx2p3h1A | 72B2FbxmM3Tpx4iruzbCNjmKthjZ8bnWvmZ7H1Zi9tSJ | 3Frvk1RMRqRpmGrH1HAZaJX779qdDkC5JBNcbT2Q8km1 |              0 | watcher04.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 6T9R5Wp6DntYVyKKic4Hqr1JxbMeQPoyVuwyRSpKF9DN |                 312780187785 |            31278018778500000000 | 2024-12-16
 14:30:44.568748+02
 11464503 | 4v3JiKvC2Ps6hmpQajjDow9LwS85ED2HYMoMjzTDsHFG | 6bxJAa1wu6FjiF8ZJKZ8nCVqNNkeSYWyQs3DVTsnkyTU | 2ADWym2TGc8AT8Aa1FkyWpe8GBZ5xZdYGQhfxQq5Yyf4 |              0 | watcher03.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | 6LU7pxJ79o2FBPhH2YWRmLJbGZfYumbm2CuToXd6ASMi |                 310348318320 |            31034831832000000000 | 2024-12-16
 14:30:41.874203+02
 11463854 | 9vtQHhvpox1MF5ykYZAc2Kxj35Ts3gLcAfmhY6tbQxox | J41bcnau1jWKDhRjj4tku177PhVCjQrKYGn4h7mFwiX  | Hat7uzfx4UpRiZCqf112Fkr1gfKn3eneigZ6bMk3FoNv |              0 | watcher02.ref
-watchdog.near  | v2.ref-finance.near | SUCCESS_RECEIPT_ID | FgcAtNqJXZQRpL8whAv2jEEGmWxCFFNYssk6Ptcq7at5 |                 312446401780 |            31244640178000000000 | 2024-12-16
events3=# EXPLAIN ANALYZE SELECT * FROM new_table WHERE receiver_account_id = 'v2.ref-finance.near' ORDER BY id DESC LIMIT 50;
                                                                                                         QUERY PLAN                                                                    
                                      
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
--------------------------------------
 Limit  (cost=217803.45..217803.58 rows=50 width=256) (actual time=175.363..175.372 rows=50 loops=1)
   ->  Sort  (cost=217803.45..218963.45 rows=464000 width=256) (actual time=171.020..171.027 rows=50 loops=1)
         Sort Key: _hyper_7_84_chunk.id DESC
         Sort Method: top-N heapsort  Memory: 53kB
         ->  Append  (cost=360.85..202389.71 rows=464000 width=256) (actual time=3.889..167.343 rows=40108 loops=1)
               ->  Custom Scan (DecompressChunk) on _hyper_7_84_chunk  (cost=360.85..24537.59 rows=68000 width=255) (actual time=3.888..20.437 rows=4472 loops=1)
                     ->  Index Scan using compress_hyper_8_94_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_94_chunk  (cost=0.43..24537.59 rows=68 width=394) (actual
 time=3.880..19.817 rows=47 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_85_chunk  (cost=512.44..25622.02 rows=50000 width=256) (actual time=4.425..21.047 rows=4137 loops=1)
                     ->  Index Scan using compress_hyper_8_95_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_95_chunk  (cost=0.43..25622.02 rows=50 width=394) (actual
 time=4.423..20.468 rows=52 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_86_chunk  (cost=521.59..26601.31 rows=51000 width=256) (actual time=3.890..21.456 rows=4058 loops=1)
                     ->  Index Scan using compress_hyper_8_96_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_96_chunk  (cost=0.55..26601.31 rows=51 width=394) (actual
 time=3.887..20.887 rows=57 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_87_chunk  (cost=413.14..26440.79 rows=64000 width=256) (actual time=4.450..21.436 rows=3746 loops=1)
                     ->  Index Scan using compress_hyper_8_97_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_97_chunk  (cost=0.55..26440.79 rows=64 width=394) (actual
 time=4.448..20.873 rows=60 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_88_chunk  (cost=443.72..23517.15 rows=53000 width=256) (actual time=3.530..19.350 rows=6261 loops=1)
                     ->  Index Scan using compress_hyper_8_98_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_98_chunk  (cost=0.43..23517.15 rows=53 width=395) (actual
 time=3.528..18.576 rows=50 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_89_chunk  (cost=441.89..20327.09 rows=46000 width=255) (actual time=3.407..17.203 rows=6383 loops=1)
                     ->  Index Scan using compress_hyper_8_99_chunk_signer_account_id_receiver_accoun_idx on compress_hyper_8_99_chunk  (cost=0.42..20327.09 rows=46 width=394) (actual
 time=3.391..16.361 rows=47 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_90_chunk  (cost=568.59..25017.86 rows=44000 width=257) (actual time=3.808..20.657 rows=4234 loops=1)
                     ->  Index Scan using compress_hyper_8_100_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_100_chunk  (cost=0.43..25017.86 rows=44 width=394) (actua
l time=3.805..20.070 rows=39 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_91_chunk  (cost=446.94..24581.49 rows=55000 width=256) (actual time=3.754..21.015 rows=5444 loops=1)
                     ->  Index Scan using compress_hyper_8_101_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_101_chunk  (cost=0.43..24581.49 rows=55 width=394) (actua
l time=3.752..20.271 rows=53 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
               ->  Custom Scan (DecompressChunk) on _hyper_7_92_chunk  (cost=103.77..3424.41 rows=33000 width=257) (actual time=0.566..3.369 rows=1373 loops=1)
                     ->  Index Scan using compress_hyper_8_102_chunk_signer_account_id_receiver_accou_idx on compress_hyper_8_102_chunk  (cost=0.42..3424.41 rows=33 width=396) (actual
 time=0.563..3.115 rows=33 loops=1)
                           Index Cond: (receiver_account_id = 'v2.ref-finance.near'::text)
 Planning Time: 0.920 ms
 JIT:
   Functions: 19
   Options: Inlining false, Optimization false, Expressions true, Deforming true
   Timing: Generation 0.496 ms, Inlining 0.000 ms, Optimization 0.220 ms, Emission 4.121 ms, Total 4.838 ms
 Execution Time: 175.976 ms
(38 rows)
