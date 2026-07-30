use minisign_verify::{PublicKey, Signature};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::{env, fs, path::Path};

fn main() {
    if let Err(message) = verify() {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

fn verify() -> Result<(), &'static str> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() != 4 {
        return Err("Použití: updater-signature-verifier <veřejný-klíč> <balíček> <podpis>");
    }
    let public_key = PublicKey::from_file(Path::new(&arguments[1]))
        .map_err(|_| "Veřejný klíč nelze načíst.")?;
    let artifact = fs::read(&arguments[2]).map_err(|_| "Aktualizační balíček nelze načíst.")?;
    let encoded_signature = fs::read_to_string(&arguments[3])
        .map_err(|_| "Podpis aktualizačního balíčku nelze načíst.")?;
    let signature_text = STANDARD.decode(encoded_signature.trim())
        .ok().and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or(encoded_signature);
    let signature = Signature::decode(&signature_text)
        .map_err(|_| "Podpis aktualizačního balíčku nelze načíst.")?;
    public_key.verify(&artifact, &signature, false)
        .map_err(|_| "Podpis aktualizačního balíčku není platný.")
}
