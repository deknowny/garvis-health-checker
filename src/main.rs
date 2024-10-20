use grammers_client::InputMessage;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};



const RECONNECtiON_POLICY: &grammers_mtsender::FixedReconnect = &grammers_mtsender::FixedReconnect {
    attempts: 30000,
    delay: std::time::Duration::from_secs(3)
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    better_panic::install();
    // console_subscriber::init();
    setup_tracer();
    tracing::info!("Run userbot image");

    let client = loop {
        let session_path = "./health-checker.grammers";
        let client = match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            grammers_client::Client::connect(grammers_client::Config {
                session: grammers_session::Session::load_file_or_create(&session_path)?,
                api_id: garvis_health_check::envconf::TG_API_ID.clone(),
                api_hash: garvis_health_check::envconf::TG_API_HASH.clone(),
                params: grammers_client::InitParams {
                    catch_up: false,
                    update_queue_limit: Some(100),
                    reconnection_policy: RECONNECtiON_POLICY,
                    ..Default::default()
                },
            }
        )).await {
            Ok(Ok(client)) => client,
            bad => {
                tracing::error!(bad = ?bad, "Cannot connect to the client, retry in 10 seconds");
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        };


        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            client.is_authorized()
        ).await {
            Ok(Ok(is_auth_value)) => {
                if !is_auth_value {
                    tracing::info!("Client is not authed, do sign in...");
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(30),
                        client.request_login_code(&garvis_health_check::envconf::ACCOUNT_PHONE.clone())
                    ).await {
                        Ok(Ok(login_token)) => {
                            tracing::info!("Enter auth code");
                            let mut auth_code = String::new();
                            std::io::stdin().read_line(&mut auth_code).unwrap();
                            match client.sign_in(&login_token, &auth_code).await {
                                Ok(_client_user) => {},
                                Err(grammers_client::SignInError::PasswordRequired(ptoken)) => {
                                    tracing::info!("Enter 2FA password");
                                    let mut twofa_password = String::new();
                                    std::io::stdin().read_line(&mut twofa_password).unwrap();

                                    match client.check_password(ptoken, twofa_password).await {
                                        Ok(_client_user) => {},
                                        Err(err) => {
                                            tracing::error!(err = ?err, "Cannot sign in, try again in 5 seconds");
                                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                            continue;
                                        },
                                    }

                                },
                                Err(err) => {
                                    tracing::error!(err = ?err, "Cannot sign in, try again in 5 seconds");
                                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                    continue;
                                }
                            }

                            tracing::info!("Client is signed in successfully");
                            if let Err(err) = client.session().save_to_file(&session_path) {
                                tracing::error!(err = ?err, "Could not save session file");
                            }
                            break client;
                        },
                        bad => {
                            tracing::error!("Cannot bot sign in, try again in 10 seconds");
                            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                            continue;
                        }
                    }
                }
            },
            bad => {
                tracing::error!(bad = ?bad, "Cannot check bot session auth, try again in 10 seconds");
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        }
    };

    loop {
        let lkbot_chat = match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            client.resolve_username("lkgarvis2bot")
        ).await {
            Ok(Ok(x)) => match x {
                Some(y) => y,
                None => {
                    tracing::error!("No such bot username. Try again in 10 seconds.");
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    continue;
                },
            },
            bad => {
                tracing::error!(bad = ?bad, "Cannot resolve bot username. Try again in 10 seconds.");
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        };

        let sent_ping = match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            client.send_message(
                lkbot_chat.pack(),
                InputMessage::text(".ping")
            )
        ).await {
            Ok(Ok(x)) => x,
            bad => {
                tracing::error!(bad = ?bad, "Cannot send ping");
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        };
        let sent_menu = match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            client.send_message(
                lkbot_chat.pack(),
                InputMessage::text("/menu")
            )
        ).await {
            Ok(Ok(x)) => x,
            bad => {
                tracing::error!(bad = ?bad, "Cannot send pong");
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        };

        tracing::info!(wait_seconds = garvis_health_check::envconf::ALIVE_PATIENCE.clone(), "Wait before Garvis responds");
        tokio::time::sleep(std::time::Duration::from_secs(garvis_health_check::envconf::ALIVE_PATIENCE.clone()))
            .await;

        let mut should_restart = false;

        let pong_message = match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            client.get_messages_by_id(lkbot_chat.pack(), &[sent_ping.id()])
        ).await {
            Ok(Ok(x)) => {
                match x.get(0) {
                    Some(Some(y)) => y.clone(),
                    bad => {
                        tracing::error!(bad = ?bad, "No pong message");
                        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                        continue;
                    }
                }
            },
            bad => {
                tracing::error!(bad = ?bad, "Cannot retrive pong");
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        };

        
        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            client.get_messages_by_id(lkbot_chat.pack(), &[sent_menu.id() + 1])
        ).await {
            Ok(Ok(x)) => {
                match x.get(0) {
                    Some(Some(_)) => {},
                    bad => {
                        tracing::error!(bad = ?bad, "Garvis does not send menu");
                        should_restart = true;
                    }
                }
            },
            bad => {
                tracing::error!(bad = ?bad, "Cannot respose menu message");
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }
        };


        if pong_message.text() != "Понг!" {
            tracing::warn!("Garvis does not respond to .ping");
            should_restart = true;
        }



        if should_restart {
            tracing::warn!("Garvis is down. Restart");
            let restart_result = tokio::process::Command::new("zsh")
                .args(&["-c", "restart_garvis"])
                .output()
                .await
            ;
            match restart_result {
                Ok(restart_output) => {
                    if restart_output.status.success() {
                        tracing::info!("Garvis restarted")
                    } else {
                        let stderr = String::from_utf8_lossy(&restart_output.stderr).to_string();
                        tracing::error!(code = ?restart_output.status, stderr = stderr, "Error due restart command invocation");
                    }
                    
                },
                Err(err) => {
                    tracing::error!(err = ?err, "Cannot invoke restart command");
                    
                },
            }
        }
        
        

        tracing::info!(next_check_after = garvis_health_check::envconf::HEALTH_CHECK_PERIOD.clone(), "Garvis works");
        tokio::time::sleep(std::time::Duration::from_secs(garvis_health_check::envconf::HEALTH_CHECK_PERIOD.clone()))
            .await;

    }

    Ok(())
}



fn setup_tracer() {
    #[cfg(debug_assertions)]
    let userbot_level = "debug";

    #[cfg(not(debug_assertions))]
    let userbot_level = "info";

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
}
