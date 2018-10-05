extern crate pbkdf2;

// // Hash the password
pub fn hash_password(password: &str) -> String {
    let hash_password : String;
    match pbkdf2::pbkdf2_simple(&password, 1000) {
        Ok(_hash) => {
            hash_password = _hash;
        },
        Err(_hash) => {
            hash_password = "".to_string();
        }
    }
    hash_password
}

// Check the password hash against the password provided
pub fn check_password(password: &str, hashed_password: &str) -> bool {
    match pbkdf2::pbkdf2_check(password, hashed_password) {
        Ok(_x) => true,
        Err(_x) => false,
    }
}