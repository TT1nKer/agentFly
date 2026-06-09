#[cfg(test)]
mod cross_tests {
    use agent_bridge::crypto::*;

    #[test]
    fn test_verify_dart_signature_in_rust() {
        let public_key_b64 = "Vv4XiufITes6t64iSZ7lKAvmGsdxH4R9vGhXoNCNCkM=";
        let signing_string = "v1\n\
            message_id=msg_cross_001\n\
            device_id=device_96164d55\n\
            type=session.input\n\
            timestamp_ms=1781000000000\n\
            nonce=AAECAwQFBgcICQoLDA0ODw==\n\
            seq=42\n\
            payload_sha256=e7451bc81415df17be3a6a4e965428d0c9efda81d4a5c41d8992a51d795327ef";
        let signature_b64 = "uSdRAqmSdi7WuDrkKaXggZ51Ju2SAKD28FAYQgJBsST7sLijzgvGKuhgVl3OGPYoneFXAhz+k7wuWasq8nfRCA==";

        let vk = public_key_from_base64(public_key_b64)
            .expect("Should parse Dart public key");

        let signature = signature_from_base64(signature_b64)
            .expect("Should parse Dart signature");

        let result = verify(&vk, signing_string, &signature);
        assert!(result.is_ok(), "Dart signature should verify in Rust: {:?}", result);
    }

    #[test]
    fn test_rust_signature_verified_in_dart_format() {
        let (sk, vk) = generate_keypair();
        let vk_b64 = public_key_to_base64(&vk);

        let _payload = serde_json::json!({"session_id": "cross_test", "content": "hello from rust"});
        let payload_sha256 = "e7451bc81415df17be3a6a4e965428d0c9efda81d4a5c41d8992a51d795327ef";

        let signing_string = build_signing_string(
            "msg_cross_002",
            "device_rust_01",
            "session.input",
            1781000000000,
            "AAECAwQFBgcICQoLDA0ODw==",
            42,
            payload_sha256,
        );

        let signature = sign(&sk, &signing_string);
        let result = verify(&vk, &signing_string, &signature);
        assert!(result.is_ok(), "Rust self-verify should work: {:?}", result);

        println!("Rust public key: {}", vk_b64);
        println!("Rust signature: {}", signature_to_base64(&signature));
    }
}
