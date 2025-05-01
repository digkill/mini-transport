use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Result;
use quinn::{Endpoint, ServerConfig};
use tokio::net::ToSocketAddrs;
use rcgen::generate_simple_self_signed;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use bytes::BytesMut; // Убрали Bytes, оставили только BytesMut, если он используется

pub async fn start_quic_server<A: ToSocketAddrs>(addr: SocketAddr) -> Result<()> {
    // ── 1. Генерируем самоподписанный сертификат ───────────────
    let cert = generate_simple_self_signed(vec!["localhost".into()])?;
    let cert_der = CertificateDer::from(cert.cert); // Прямо используем Certificate из rcgen
    let key_der = PrivateKeyDer::Pkcs8(cert.key_pair.serialize_der().into()); // Используем PKCS#8

    // ── 2. Собираем конфиг QUIC-сервера (rustls внутри) ─────
    let mut server_config = ServerConfig::with_single_cert(
        vec![cert_der], // Передаем вектор CertificateDer
        key_der,        // Передаем PrivateKeyDer
    )?;
    server_config.transport = Arc::new(quinn::TransportConfig::default());

    // ── 3. Создаем QUIC-эндпоинт ───────────────────────────
    let endpoint = Endpoint::server(server_config, addr)?;

    // ── 4. Обрабатываем входящие подключения ───────────────
    while let Some(conn) = endpoint.accept().await {
        tokio::spawn(async move {
            if let Ok(new_conn) = conn.await {
                while let Ok((mut send, mut recv)) = new_conn.accept_bi().await {
                    let mut data = Vec::new();
                    while let Some(chunk) = recv.read_chunk(usize::MAX, true).await.unwrap() {
                        data.extend_from_slice(&chunk.bytes);
                    }
                    send.write_all(&data).await.unwrap();
                }
            }
        });
    }
    endpoint.wait_idle().await;
    Ok(())
}