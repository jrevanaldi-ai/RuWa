// Re-export everything from wacore-noise-ng
pub use wacore_noise_ng::{
    EdgeRoutingError, HandshakeError, HandshakeResult as Result, HandshakeState, HandshakeUtils,
    MAX_EDGE_ROUTING_LEN, NoiseCipher, NoiseHandshake, WA_CERT_PUB_KEY,
    build_edge_routing_preintro, build_handshake_header, generate_iv,
};
