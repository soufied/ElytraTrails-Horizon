fn elytratrails_settings_page(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    buttons: &mut Vec<String>,
) {
    use horizon_client_render::gfx::native_render::elytratrails as m;
    if mod_id == m::ID {
        buttons.extend(game.elytratrails_page());
    }
}
