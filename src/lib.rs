pub mod game;
pub mod hash_list;
pub mod metadata;
pub mod rpkg_tool;

#[cfg(feature = "resourcelib")]
pub mod resourcelib;

#[cfg(feature = "game_detection")]
pub mod game_detection;

#[cfg(feature = "rune")]
pub fn rune_install(ctx: &mut rune::Context, allow_dangerous: bool) -> Result<(), rune::ContextError> {
	ctx.install(game::rune_module()?)?;
	ctx.install(metadata::rune_module()?)?;
	ctx.install(rpkg_tool::rune_module()?)?;
	ctx.install(hash_list::rune_module()?)?;

	#[cfg(feature = "resourcelib")]
	ctx.install(resourcelib::rune_module()?)?;

	#[cfg(feature = "game_detection")]
	if allow_dangerous {
		ctx.install(game_detection::rune_module()?)?;
	}

	Ok(())
}
