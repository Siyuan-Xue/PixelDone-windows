use std::{fs, path::PathBuf};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use minisign_verify::{PublicKey, Signature};

#[test]
#[ignore = "requires the formal NSIS artifact"]
fn formal_nsis_signature_matches_embedded_public_key() {
    let artifact = PathBuf::from(
        "target/x86_64-pc-windows-msvc/release/bundle/nsis/PixelDone_3.1.3_x64-setup.exe",
    );
    let signature_path = artifact.with_extension("exe.sig");
    let public_wrapper = fs::read_to_string("signing/pixeldone-updater.key.pub").unwrap();
    let signature_wrapper = fs::read_to_string(signature_path).unwrap();
    let public_text = String::from_utf8(STANDARD.decode(public_wrapper.trim()).unwrap()).unwrap();
    let signature_text =
        String::from_utf8(STANDARD.decode(signature_wrapper.trim()).unwrap()).unwrap();
    let public_key = PublicKey::decode(&public_text).unwrap();
    let signature = Signature::decode(&signature_text).unwrap();
    let bytes = fs::read(artifact).unwrap();
    public_key.verify(&bytes, &signature, false).unwrap();
}
