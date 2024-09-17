pub mod game;
pub mod metadata;
pub mod rpkg_tool;

#[cfg(feature = "resourcelib")]
pub mod resourcelib;

#[cfg(feature = "hash_list")]
pub mod hash_list;

#[cfg(feature = "game_detection")]
pub mod game_detection;
