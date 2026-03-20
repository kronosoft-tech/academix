use bcrypt::{hash, verify, DEFAULT_COST};

fn main() {
    let password = "admin123";
    
    let hashed = hash(password, DEFAULT_COST).expect("Failed to hash");
    println!("Password: {}", password);
    println!("Hash: {}", hashed);
    
    // Verificar que funciona
    let verify_result = verify(password, &hashed).expect("Failed to verify");
    println!("Verify result: {}", verify_result);
}
