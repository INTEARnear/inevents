#[cfg(feature = "impl")]
use intear_events::events;

#[cfg(feature = "impl")]
use events::{
    nft::{nft_burn::NftBurnEvent, nft_mint::NftMintEvent, nft_transfer::NftTransferEvent},
    potlock::{
        potlock_donation::PotlockDonationEvent, potlock_pot_donation::PotlockPotDonationEvent,
        potlock_pot_project_donation::PotlockPotProjectDonationEvent,
    },
    price::{price_pool::PricePoolEvent, price_token::PriceTokenEvent},
    trade::{
        trade_pool::TradePoolEvent, trade_pool_change::TradePoolChangeEvent,
        trade_swap::TradeSwapEvent,
    },
};
#[cfg(feature = "impl")]
use inevents::{
    create_events,
    modules::{
        http_server::HttpServer, redis_to_postgres::RedisToPostgres,
        websocket_server::WebsocketServer, EventModule,
    },
};
#[cfg(feature = "impl")]
use intear_events::events::{
    log::{log_nep297::LogNep297Event, log_text::LogTextEvent},
    newcontract::{meme_cooking_meme::NewMemeCookingMemeEvent, nep141::NewContractNep141Event},
    socialdb::index::SocialDBIndexEvent,
    tps::{block_info::BlockInfoEvent, moretps_claims::MoreTpsClaimEvent},
};

#[cfg(feature = "impl")]
#[tokio::main]
async fn main() {
    use intear_events::events::ft::ft_burn::FtBurnEvent;
    use intear_events::events::ft::ft_mint::FtMintEvent;
    use intear_events::events::ft::ft_transfer::FtTransferEvent;
    use intear_events::events::newcontract::meme_cooking_token::NewMemeCookingTokenEvent;
    use intear_events::events::trade::liquidity_pool::LiquidityPoolEvent;
    use intear_events::events::trade::memecooking_deposit::MemeCookingDepositEvent;
    use intear_events::events::trade::memecooking_withdraw::MemeCookingWithdrawEvent;
    use intear_events::events::transactions::tx_receipt::TxReceiptEvent;
    use intear_events::events::transactions::tx_transaction::TxTransactionEvent;

    dotenvy::dotenv().ok();
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .env()
        .init()
        .unwrap();

    let module_names = std::env::args().skip(1).collect::<Vec<String>>();
    if module_names.is_empty() {
        log::error!("No modules specified. Available modules: 'http-server', 'websocket-server', 'redis-to-postgres', 'all'. Usage: `./inevents [module]...`");
        return;
    }
    let module_names = if module_names.contains(&"all".to_string()) {
        vec![
            "http-server".to_string(),
            "websocket-server".to_string(),
            "redis-to-postgres".to_string(),
        ]
    } else {
        module_names
    };
    log::info!("Running {}", module_names.join(", "));

    create_events!(Events:
        NftMintEvent,
        NftBurnEvent,
        NftTransferEvent,
        PotlockDonationEvent,
        PotlockPotProjectDonationEvent,
        PotlockPotDonationEvent,
        TradePoolEvent,
        TradeSwapEvent,
        TradePoolChangeEvent,
        PricePoolEvent,
        PriceTokenEvent,
        NewContractNep141Event,
        SocialDBIndexEvent,
        LogTextEvent,
        LogNep297Event,
        NewMemeCookingMemeEvent,
        BlockInfoEvent,
        MoreTpsClaimEvent,
        MemeCookingDepositEvent,
        MemeCookingWithdrawEvent,
        NewMemeCookingTokenEvent,
        TxTransactionEvent,
        TxReceiptEvent,
        FtMintEvent,
        FtTransferEvent,
        FtBurnEvent,
        LiquidityPoolEvent,
    );

    let mut futures = Vec::new();
    for module_name in module_names {
        match module_name.as_str() {
            "http-server" => {
                futures.push(
                    HttpServer::new(Some(
                        "https://docs.intear.tech/docs/events-api/".to_string(),
                    ))
                    .start::<Events>(),
                );
            }
            "websocket-server" => {
                futures.push(
                    WebsocketServer::new(Some(
                        "https://docs.intear.tech/docs/events-api/".to_string(),
                    ))
                    .start::<Events>(),
                );
            }
            "redis-to-postgres" => {
                futures.push(RedisToPostgres.start::<Events>());
            }
            _ => {
                log::error!("Unknown module: {module_name}");
                return;
            }
        }
    }
    let results = futures::future::join_all(futures).await;
    for result in results {
        if let Err(e) = result {
            log::error!("Error in module: {e:?}");
        }
    }
}

#[cfg(not(feature = "impl"))]
fn main() {
    log::error!("This binary was compiled without the 'impl' feature. Please enable the 'impl' feature to run the binary.");
}
