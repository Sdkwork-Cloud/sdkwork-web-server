use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    time::Duration,
};

use axum::extract::connect_info::Connected;
use sdkwork_webserver_core::{ProxyProtocolConfig, ProxyProtocolVersion};
use tokio::{io::AsyncReadExt, net::TcpStream, time::timeout};

use super::{connection_limit::ConnectionLimitedStream, real_ip::is_trusted};

const V2_SIGNATURE: [u8; 12] = *b"\r\n\r\n\0\r\nQUIT\n";
const V1_MAX_BYTES: usize = 107;

#[derive(Debug, Clone, Copy)]
pub(super) struct DownstreamConnectionInfo {
    pub transport_peer: SocketAddr,
    pub client_peer: SocketAddr,
    pub proxy_protocol: Option<ProxyProtocolVersion>,
}

impl Connected<Self> for DownstreamConnectionInfo {
    fn connect_info(info: Self) -> Self {
        info
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProxyProtocolError {
    UntrustedPeer,
    Timeout,
    Invalid,
    UnsupportedVersion,
    Io,
}

pub(super) async fn resolve_connection_info(
    stream: &mut ConnectionLimitedStream<TcpStream>,
    transport_peer: SocketAddr,
    policy: Option<&ProxyProtocolConfig>,
) -> Result<DownstreamConnectionInfo, ProxyProtocolError> {
    let Some(policy) = policy else {
        return Ok(DownstreamConnectionInfo {
            transport_peer,
            client_peer: transport_peer,
            proxy_protocol: None,
        });
    };
    if !is_trusted(transport_peer.ip(), &policy.trusted_source_cidrs) {
        return Err(ProxyProtocolError::UntrustedPeer);
    }
    timeout(
        Duration::from_millis(policy.timeout_ms),
        parse_header(stream, transport_peer, policy),
    )
    .await
    .map_err(|_| ProxyProtocolError::Timeout)?
}

async fn parse_header(
    stream: &mut ConnectionLimitedStream<TcpStream>,
    transport_peer: SocketAddr,
    policy: &ProxyProtocolConfig,
) -> Result<DownstreamConnectionInfo, ProxyProtocolError> {
    let mut prefix = [0_u8; 12];
    read_exact(stream, &mut prefix).await?;
    if prefix == V2_SIGNATURE {
        if !policy.versions.contains(&ProxyProtocolVersion::V2) {
            return Err(ProxyProtocolError::UnsupportedVersion);
        }
        parse_v2(stream, transport_peer, policy.max_header_bytes).await
    } else if prefix.starts_with(b"PROXY ") {
        if !policy.versions.contains(&ProxyProtocolVersion::V1) {
            return Err(ProxyProtocolError::UnsupportedVersion);
        }
        parse_v1(stream, transport_peer, prefix).await
    } else {
        Err(ProxyProtocolError::Invalid)
    }
}

async fn parse_v1(
    stream: &mut ConnectionLimitedStream<TcpStream>,
    transport_peer: SocketAddr,
    prefix: [u8; 12],
) -> Result<DownstreamConnectionInfo, ProxyProtocolError> {
    let mut line = [0_u8; V1_MAX_BYTES];
    line[..prefix.len()].copy_from_slice(&prefix);
    let mut used = prefix.len();
    loop {
        if let Some(end) = line[..used].windows(2).position(|pair| pair == b"\r\n") {
            return parse_v1_line(&line[..end], transport_peer);
        }
        if used == line.len() {
            return Err(ProxyProtocolError::Invalid);
        }
        let available = stream.peek(&mut line[used..]).await.map_err(classify_io)?;
        if available == 0 {
            return Err(ProxyProtocolError::Invalid);
        }
        let visible = used + available;
        let consume = line[..visible]
            .windows(2)
            .position(|pair| pair == b"\r\n")
            .map_or(available, |end| end + 2 - used);
        read_exact(stream, &mut line[used..used + consume]).await?;
        used += consume;
    }
}

fn parse_v1_line(
    line: &[u8],
    transport_peer: SocketAddr,
) -> Result<DownstreamConnectionInfo, ProxyProtocolError> {
    if line == b"PROXY UNKNOWN" || line.starts_with(b"PROXY UNKNOWN ") {
        return Ok(info(
            transport_peer,
            transport_peer,
            ProxyProtocolVersion::V1,
        ));
    }
    let line = std::str::from_utf8(line).map_err(|_| ProxyProtocolError::Invalid)?;
    let mut fields = line.split(' ');
    if fields.next() != Some("PROXY") {
        return Err(ProxyProtocolError::Invalid);
    }
    let family = fields.next().ok_or(ProxyProtocolError::Invalid)?;
    let source = fields.next().ok_or(ProxyProtocolError::Invalid)?;
    let destination = fields.next().ok_or(ProxyProtocolError::Invalid)?;
    let source_port = parse_port(fields.next().ok_or(ProxyProtocolError::Invalid)?)?;
    let _destination_port = parse_port(fields.next().ok_or(ProxyProtocolError::Invalid)?)?;
    if fields.next().is_some() {
        return Err(ProxyProtocolError::Invalid);
    }
    let source = match family {
        "TCP4" => {
            let source = source
                .parse::<Ipv4Addr>()
                .map_err(|_| ProxyProtocolError::Invalid)?;
            destination
                .parse::<Ipv4Addr>()
                .map_err(|_| ProxyProtocolError::Invalid)?;
            IpAddr::V4(source)
        }
        "TCP6" => {
            let source = source
                .parse::<Ipv6Addr>()
                .map_err(|_| ProxyProtocolError::Invalid)?;
            destination
                .parse::<Ipv6Addr>()
                .map_err(|_| ProxyProtocolError::Invalid)?;
            IpAddr::V6(source)
        }
        _ => return Err(ProxyProtocolError::Invalid),
    };
    Ok(info(
        transport_peer,
        SocketAddr::new(source, source_port),
        ProxyProtocolVersion::V1,
    ))
}

async fn parse_v2(
    stream: &mut ConnectionLimitedStream<TcpStream>,
    transport_peer: SocketAddr,
    maximum_bytes: usize,
) -> Result<DownstreamConnectionInfo, ProxyProtocolError> {
    let mut fixed = [0_u8; 4];
    read_exact(stream, &mut fixed).await?;
    if fixed[0] >> 4 != 2 {
        return Err(ProxyProtocolError::Invalid);
    }
    let command = fixed[0] & 0x0f;
    let family = fixed[1];
    let length = u16::from_be_bytes([fixed[2], fixed[3]]) as usize;
    if 16_usize
        .checked_add(length)
        .is_none_or(|total| total > maximum_bytes)
    {
        return Err(ProxyProtocolError::Invalid);
    }
    if command == 0 {
        discard_exact(stream, length).await?;
        return Ok(info(
            transport_peer,
            transport_peer,
            ProxyProtocolVersion::V2,
        ));
    }
    if command != 1 {
        return Err(ProxyProtocolError::Invalid);
    }
    let (address_bytes, client_peer) = match family {
        0x11 if length >= 12 => {
            let mut address = [0_u8; 12];
            read_exact(stream, &mut address).await?;
            let source = Ipv4Addr::new(address[0], address[1], address[2], address[3]);
            let port = u16::from_be_bytes([address[8], address[9]]);
            (12, SocketAddr::new(IpAddr::V4(source), port))
        }
        0x21 if length >= 36 => {
            let mut address = [0_u8; 36];
            read_exact(stream, &mut address).await?;
            let mut source = [0_u8; 16];
            source.copy_from_slice(&address[..16]);
            let port = u16::from_be_bytes([address[32], address[33]]);
            (
                36,
                SocketAddr::new(IpAddr::V6(Ipv6Addr::from(source)), port),
            )
        }
        _ => return Err(ProxyProtocolError::Invalid),
    };
    discard_exact(stream, length - address_bytes).await?;
    Ok(info(transport_peer, client_peer, ProxyProtocolVersion::V2))
}

fn info(
    transport_peer: SocketAddr,
    client_peer: SocketAddr,
    version: ProxyProtocolVersion,
) -> DownstreamConnectionInfo {
    DownstreamConnectionInfo {
        transport_peer,
        client_peer,
        proxy_protocol: Some(version),
    }
}

fn parse_port(value: &str) -> Result<u16, ProxyProtocolError> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ProxyProtocolError::Invalid);
    }
    value.parse().map_err(|_| ProxyProtocolError::Invalid)
}

async fn discard_exact(
    stream: &mut ConnectionLimitedStream<TcpStream>,
    mut bytes: usize,
) -> Result<(), ProxyProtocolError> {
    let mut buffer = [0_u8; 256];
    while bytes != 0 {
        let take = bytes.min(buffer.len());
        read_exact(stream, &mut buffer[..take]).await?;
        bytes -= take;
    }
    Ok(())
}

async fn read_exact<I: tokio::io::AsyncRead + Unpin>(
    stream: &mut I,
    buffer: &mut [u8],
) -> Result<(), ProxyProtocolError> {
    stream
        .read_exact(buffer)
        .await
        .map(|_| ())
        .map_err(classify_io)
}

fn classify_io(error: io::Error) -> ProxyProtocolError {
    if error.kind() == io::ErrorKind::UnexpectedEof {
        ProxyProtocolError::Invalid
    } else {
        ProxyProtocolError::Io
    }
}
