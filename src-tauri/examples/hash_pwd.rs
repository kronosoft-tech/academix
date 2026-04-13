// Quick utility to hash password for seeding
// Run: cd src-tauri && cargo run --example hash_pwd

fn main() {
    let password = "123456!";
    let hash = bcrypt::hash(password, 10).unwrap();
    println!("Hash: {}", hash);
}
