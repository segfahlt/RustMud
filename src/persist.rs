use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};

use crate::mob::Equipment;
use crate::world::{MonsterTemplate, ObjectInstance, PlayerLocation};

// --- Permissions ---

/// Non-exclusive permission flags stored on each account.
/// Admins implicitly satisfy any permission check — see `has_perm()`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Player,   // baseline — all accounts
    Remort,   // has completed a remort cycle
    Builder,  // can create/edit world content in-game
    Monitor,  // can observe other players
    Dev,      // developer access (reboot, diagnostics)
    Admin,    // full access; implicitly satisfies any permission check
}

/// Returns true if the set contains `perm` or the Admin wildcard.
pub fn has_perm(permissions: &HashSet<Permission>, perm: Permission) -> bool {
    permissions.contains(&Permission::Admin) || permissions.contains(&perm)
}

fn default_permissions() -> HashSet<Permission> {
    [Permission::Player].into()
}

// --- Account file (one per account, permanent) ---

/// A character name + id stored inside the account file,
/// so the character-select screen doesn't need to open each char file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CharacterRef {
    pub id:   String,  // = name.to_lowercase()
    pub name: String,  // display name, original case
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountFile {
    pub username:      String,
    pub password_hash: String,
    #[serde(default)]
    pub characters:    Vec<CharacterRef>,
}

// --- Character file (one per character, permanent) ---

#[derive(Debug, Serialize, Deserialize)]
pub struct CharacterFile {
    pub id:         String,
    pub account_id: String,
    pub name:       String,
    // Where the character returns on a reboot-refresh or explicit /home command.
    // None → START_LOC until explicitly set.
    #[serde(default)]
    pub home_location: Option<PlayerLocation>,
    #[serde(default = "default_permissions")]
    pub permissions: HashSet<Permission>,
}

// --- Save data (written on shutdown, read on startup) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSave {
    pub location:   PlayerLocation,
    pub health:     u32,
    pub max_health: u32,
    #[serde(default)]
    pub inventory:  Vec<ObjectInstance>,
    #[serde(default)]
    pub equipment:  Equipment,
    /// Last Area visited — used to return players when exiting a Room cluster.
    #[serde(default)]
    pub last_area:  Option<PlayerLocation>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WorldSave {
    #[serde(default)]
    pub characters: HashMap<String, CharacterSave>,  // character_id → save
    /// Objects on Area floors, keyed by "zone_q:zone_r:area_id".
    #[serde(default)]
    pub area_objects: HashMap<String, Vec<ObjectInstance>>,
    /// Objects on Room floors, keyed by room_id.
    #[serde(default)]
    pub room_objects: HashMap<u32, Vec<ObjectInstance>>,
    /// Procedurally generated mob templates, keyed by template id.
    #[serde(default)]
    pub generated_mob_templates: HashMap<String, MonsterTemplate>,
}

// --- Password hashing (argon2id) ---

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("argon2 hash failed")
        .to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .map(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
        .unwrap_or(false)
}

// --- Account I/O ---

pub fn load_account(accounts_dir: &Path, id: &str) -> Option<AccountFile> {
    let path = accounts_dir.join(format!("{id}.json"));
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

pub fn write_account(accounts_dir: &Path, account: &AccountFile) -> io::Result<()> {
    fs::create_dir_all(accounts_dir)?;
    let path = accounts_dir.join(format!("{}.json", account.username));
    fs::write(path, serde_json::to_string_pretty(account)?)
}

// --- Character I/O ---

pub fn load_character(chars_dir: &Path, id: &str) -> Option<CharacterFile> {
    let path = chars_dir.join(format!("{id}.json"));
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

pub fn write_character(chars_dir: &Path, character: &CharacterFile) -> io::Result<()> {
    fs::create_dir_all(chars_dir)?;
    let path = chars_dir.join(format!("{}.json", character.id));
    fs::write(path, serde_json::to_string_pretty(character)?)
}

pub fn character_name_taken(chars_dir: &Path, id: &str) -> bool {
    chars_dir.join(format!("{id}.json")).exists()
}

// --- World save I/O ---

pub fn load_world_save(path: &Path) -> WorldSave {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn write_world_save(save: &WorldSave, path: &Path) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, serde_json::to_string_pretty(save)?)
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    fn make_obj(template_id: &str) -> ObjectInstance {
        ObjectInstance::new(template_id)
    }

    #[test]
    fn world_save_round_trips_area_objects() {
        let mut save = WorldSave::default();
        let key = "0:1:3".to_string();
        let objs = vec![make_obj("scrap"), make_obj("ration")];
        save.area_objects.insert(key.clone(), objs.clone());

        let json = serde_json::to_string(&save).unwrap();
        let restored: WorldSave = serde_json::from_str(&json).unwrap();
        let restored_objs = restored.area_objects.get(&key).unwrap();
        assert_eq!(restored_objs.len(), 2);
        assert_eq!(restored_objs[0].template_id, "scrap");
        assert_eq!(restored_objs[1].template_id, "ration");
    }

    #[test]
    fn world_save_round_trips_room_objects() {
        let mut save = WorldSave::default();
        let mut obj = make_obj("knife");
        obj.quantity = 3;
        save.room_objects.insert(42, vec![obj]);

        let json = serde_json::to_string(&save).unwrap();
        let restored: WorldSave = serde_json::from_str(&json).unwrap();
        let objs = restored.room_objects.get(&42).unwrap();
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0].template_id, "knife");
        assert_eq!(objs[0].quantity, 3);
    }

    #[test]
    fn world_save_preserves_bound_to() {
        let mut save = WorldSave::default();
        let mut obj = make_obj("pass");
        obj.bound_to = Some("alice".to_string());
        save.room_objects.insert(7, vec![obj]);

        let json = serde_json::to_string(&save).unwrap();
        let restored: WorldSave = serde_json::from_str(&json).unwrap();
        assert_eq!(
            restored.room_objects[&7][0].bound_to.as_deref(),
            Some("alice")
        );
    }

    #[test]
    fn world_save_defaults_to_empty_on_missing_area() {
        let save = WorldSave::default();
        assert!(save.area_objects.get("9:9:99").is_none());
    }
}
